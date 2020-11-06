import requests
import json
import config
import calendar
import sys
import time
import math
import xmltodict
import logging
import arrow
import os
from pymongo import MongoClient
from datetime import datetime, date, timedelta
import detectlanguage
from Queue import Queue
from twython import Twython
from apikeyprovider import APIKeyProvider
from cmreslogging.handlers import CMRESHandler

apiKeyProvider = APIKeyProvider(config.apiKey())
videoKeyProvider = APIKeyProvider(config.apiVideoKeys())

# logging
logger = logging.getLogger("sailing-channels-crawler")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
formatter = logging.Formatter(
    '%(asctime)s - %(name)s - %(levelname)s - %(message)s')
ch.setFormatter(formatter)
logger.addHandler(ch)

print "environment", os.getenv("ENVIRONMENT")

if os.getenv("ENVIRONMENT") == "production":
    elasticHandler = CMRESHandler(hosts=[{"host": "elasticsearch", "port": 9200}],
                                  auth_type=CMRESHandler.AuthType.NO_AUTH,
                                  es_index_name="sailing-channels-crawler")

    logger.addHandler(elasticHandler)

# social networks
twitter = Twython(config.twitter()["consumerKey"], config.twitter()[
    "consumerSecret"], config.twitter()["accessToken"], config.twitter()["accessSecret"])

# config
startChannelId = "UC5xDht2blPNWdVtl9PkDmgA"  # SailLife
maxLevels = 3
popSubsWeight = 0.5
popViewsWeight = 0.5
sailingTerms = []
blacklist = []
THREE_HOURS_IN_SECONDS = 10800
ONE_HOUR_IN_SECONDS = 3600
ONE_DAY_IN_SECONDS = 86400
ONE_WEEK_IN_SECONDS = 604800

# open mongodb connection
client = MongoClient(config.mongoDB())
db_name = "sailing-channels"
devMode = False

if len(sys.argv) != 2:
    db_name += "-dev"
    devMode = True
    logger.info("*** DEVELOPER MODE ***")

db = client[db_name]

# add sailing terms
for tt in db.sailingterms.find({}):
    sailingTerms.append(tt["_id"])

# fill blacklist
for bb in db.blacklist.find({}):
    blacklist.append(bb["_id"])

logger.info(sailingTerms)

# members
channels = {}


def deleteChannel(channelId):
    db.channels.delete_one({"_id": channelId})
    db.videos.delete_many({"channel": channelId})
    db.views.delete_many({"_id.channel": channelId})
    db.visits.delete_many({"channel": channelId})
    db.subscribers.delete_many({"_id.channel": channelId})


def storeVideoWithStats(channelId, vid):

    existingVideo = db.videos.find_one({"_id": vid["id"]}, projection=[
        "_id", "updatedAt", "publishedAt"])

    shouldUpdateVideo = True

    if existingVideo is not None and "updatedAt" in existingVideo:

        if not ("publishedAt" in existingVideo):
            existingVideo["publishedAt"] = arrow.utcnow().timestamp
            logger.warning(
                "video %s did not have a publishedAt date, tmp set to current timestamp", vid["id"])

        uploadedLaterThanThreshold = ONE_HOUR_IN_SECONDS * 3
        publishedSinceSeconds = math.fabs(
            int(existingVideo["publishedAt"]) - arrow.utcnow().timestamp)

        if publishedSinceSeconds >= ONE_WEEK_IN_SECONDS:
            uploadedLaterThanThreshold = ONE_DAY_IN_SECONDS

        if publishedSinceSeconds >= 4 * ONE_WEEK_IN_SECONDS:
            uploadedLaterThanThreshold = ONE_WEEK_IN_SECONDS

        if publishedSinceSeconds >= 6 * 4 * ONE_WEEK_IN_SECONDS:
            uploadedLaterThanThreshold = 4 * ONE_WEEK_IN_SECONDS

        updatedTimeDiff = math.fabs(int(
            existingVideo["updatedAt"]) - arrow.utcnow().timestamp)
        shouldUpdateVideo = updatedTimeDiff >= uploadedLaterThanThreshold

    if shouldUpdateVideo:

        # fetch video statistics
        key = videoKeyProvider.apiKey()
        rd = requests.get(
            "https://www.googleapis.com/youtube/v3/videos?part=snippet,statistics,status&id=" + vid["id"] + "&key=" + key)
        videoStat = rd.json()
        statistics = None

        if videoStat is None or "items" not in videoStat:
            logger.warning("could not retrieve statistic for channel %s, apikey %s, statuscode %d, content: %s",
                           channelId, key, rd.status_code, rd.content)
            return

        if len(videoStat["items"]) > 0:
            statistics = videoStat["items"][0]["statistics"]

            # store video tags
            try:
                vid["tags"] = [x.lower()
                               for x in videoStat["items"][0]["snippet"]["tags"]]
            except:
                pass

        if statistics:
            if statistics.has_key("viewCount"):
                vid["views"] = int(statistics["viewCount"])

            if statistics.has_key("likeCount"):
                vid["likes"] = int(statistics["likeCount"])

            if statistics.has_key("dislikeCount"):
                vid["dislikes"] = int(statistics["dislikeCount"])

            if statistics.has_key("commentCount"):
                vid["comments"] = int(statistics["commentCount"])

        # status of video
        status = videoStat["items"][0]["status"]

        if status["privacyStatus"] == "public":

            # prepare video for inserting into database
            dbVid = vid
            dbVid["_id"] = dbVid["id"]
            dbVid["channel"] = channelId
            dbVid["updatedAt"] = arrow.utcnow().timestamp
            del dbVid["id"]

            try:
                vidDoesNotExists = existingVideo is None

                # reasonable fresh video, post to twitter and facebook
                if vidDoesNotExists and math.fabs(int(dbVid["publishedAt"]) - time.mktime(datetime.utcnow().timetuple())) <= 15000:

                    ch = db.channels.find_one(
                        {"_id": channelId}, projection=["title"])

                    # twitter
                    try:
                        msg = "New: " + ch["title"] + " \"" + dbVid["title"] + \
                            "\" https://sailing-channels.com/#/channel/" + channelId
                        if devMode <> True:
                            twitter.update_status(status=msg)
                        else:
                            logger.warning(msg)

                    except Exception, e:
                        logger.exception(e)

                # update information in database
                db.videos.update_one({
                    "_id": dbVid["_id"]
                }, {
                    "$set": dbVid
                }, True)

                logger.info("upserted video %s for channel %s",
                            dbVid["_id"], channelId)

            except:
                pass
        else:

            # remove non public videos
            db.videos.delete_one({
                "_id": dbVid["id"]
            })

            logger.info("deleted video %s for channel %s",
                        dbVid["_id"], channelId)


def readStatistics(channelId):

    r = requests.get("https://www.googleapis.com/youtube/v3/channels?part=statistics,snippet,brandingSettings&id=" +
                     channelId + "&key=" + apiKeyProvider.apiKey())
    stats = r.json()

    if stats is None or "items" not in stats:
        return None, None, None

    return stats["items"][0]["statistics"], stats["items"][0]["snippet"], stats["items"][0]["brandingSettings"]


def getChannelPopularityIndex(channelId, subscribers, views):

    # fetch subs and views from 7 days ago
    daysAgo = date.today() - timedelta(days=2)

    daysSubs = db.subscribers.find_one({
        "_id": {
            "channel": channelId,
            "date": int(daysAgo.strftime("%Y%m%d"))
        }
    })

    daysViews = db.views.find_one({
        "_id": {
            "channel": channelId,
            "date": int(daysAgo.strftime("%Y%m%d"))
        }
    })

    popSub = 0
    popView = 0

    # calculate subscriber gain
    if subscribers > 0 and daysSubs is not None and daysSubs.has_key("subscribers"):
        popSub = math.fabs(subscribers - daysSubs["subscribers"]) / subscribers

    if views > 0 and daysViews is not None and daysViews.has_key("views"):
        popView = math.fabs(views - daysViews["views"]) / views

    return popSub, popView


def upsertChannel(subChannelId, level, readSubs=True, ignoreSailingTerm=False):

    try:

        # store this channel
        if not channels.has_key(subChannelId):

            last_crawled = db.channels.find_one(
                {"_id": subChannelId}, projection=["lastCrawl"])

            if last_crawled:
                if (datetime.now() - last_crawled["lastCrawl"]).total_seconds() < ONE_DAY_IN_SECONDS:
                    logger.info("skip %s lass crawl less than %d secs ago",
                                subChannelId, ONE_HOUR_IN_SECONDS * 3)
                    return

            stats, channel_detail, branding_settings = readStatistics(
                subChannelId)

            hasSailingTerm = False

            if stats is None:
                logger.warning(
                    "could not read stats for channel %s", subChannelId)
                return

            # check if one of the sailing terms is available
            for term in sailingTerms:
                if (term in channel_detail["title"].lower() or term in channel_detail["description"].lower()):
                    hasSailingTerm = True
                    break

            # log what happened to the channel
            logger.info("%s %s %d", subChannelId,
                        hasSailingTerm, int(stats["videoCount"]))

            if ignoreSailingTerm == True:
                hasSailingTerm = True

            # blacklisted channel
            if subChannelId in blacklist:
                hasSailingTerm = False
                deleteChannel(subChannelId)

            if int(stats["videoCount"]) > 0 and hasSailingTerm:

                pd = datetime.strptime(
                    channel_detail["publishedAt"], "%Y-%m-%dT%H:%M:%SZ")

                channels[subChannelId] = {
                    "id": subChannelId,
                    "title": channel_detail["title"],
                    "description": channel_detail["description"],
                    "publishedAt": calendar.timegm(pd.utctimetuple()),
                    "thumbnail": channel_detail["thumbnails"]["default"]["url"],
                    "subscribers": int(stats["subscriberCount"]),
                    "views": int(stats["viewCount"]),
                    "subscribersHidden": bool(stats["hiddenSubscriberCount"]),
                    "lastCrawl": datetime.now()
                }

                # add keywords if available
                try:
                    if branding_settings is not None and "channel" in branding_settings and "keywords" in branding_settings["channel"]:
                        channels[subChannelId]["keywords"] = branding_settings["channel"]["keywords"].split(
                            " ")
                except Exception, e:
                    logger.exception(e)

                # get popularity
                popSub, popView = getChannelPopularityIndex(subChannelId, int(
                    stats["subscriberCount"]), int(stats["viewCount"]))
                channels[subChannelId]["popularity"] = {
                    "subscribers": popSub,
                    "views": popView,
                    "total": popSub * popSubsWeight + popView * popViewsWeight
                }

                lotsOfText = channels[subChannelId]["description"] + " "

                # add country info if available
                if channel_detail.has_key("country"):
                    channels[subChannelId]["country"] = channel_detail["country"].lower()

                hasLanguage = False
                ch_lang = db.channels.find_one(
                    {"_id": subChannelId}, projection=["detectedLanguage"])
                if ch_lang:
                    if ch_lang.has_key("detectedLanguage"):
                        hasLanguage = True

                try:
                    useDetectLangKey = 0
                    detectlanguage.configuration.api_key = config.detectLanguage()[
                        useDetectLangKey]

                    # detect the language of the channel
                    if not hasLanguage and devMode == False:

                        channels[subChannelId]["language"] = "en"

                        runLoop = True
                        while runLoop:
                            try:
                                detectedLang = detectlanguage.detect(
                                    lotsOfText)
                                runLoop = False
                            except:
                                useDetectLangKey = useDetectLangKey + 1

                                if useDetectLangKey > len(config.detectLanguage()):
                                    runLoop = False
                                else:
                                    detectlanguage.configuration.api_key = config.detectLanguage()[
                                        useDetectLangKey]

                        # did we find a language in the text body?
                        if len(detectedLang) > 0:

                            # is the detection reliable?
                            if detectedLang[0]["isReliable"]:
                                channels[subChannelId]["language"] = detectedLang[0]["language"]
                                channels[subChannelId]["detectedLanguage"] = True
                except Exception, e:
                    logger.exception(e)

                # insert subscriber counts
                try:
                    db.subscribers.update_one({
                        "_id": {
                            "channel": subChannelId,
                            "date": int(date.today().strftime("%Y%m%d"))
                        }
                    }, {
                        "$set": {
                            "year": date.today().year,
                            "month": date.today().month,
                            "day": date.today().day,
                            "date": datetime.utcnow(),
                            "subscribers": channels[subChannelId]["subscribers"]
                        }
                    }, True)
                except Exception, e:
                    logger.exception(e)

                # insert view counts
                try:
                    db.views.update_one({
                        "_id": {
                            "channel": subChannelId,
                            "date": int(date.today().strftime("%Y%m%d"))
                        }
                    }, {
                        "$set": {
                            "year": date.today().year,
                            "month": date.today().month,
                            "day": date.today().day,
                            "date": datetime.utcnow(),
                            "views": channels[subChannelId]["views"]
                        }
                    }, True)
                except Exception, e:
                    logger.exception(e)

                # upsert data in mongodb
                try:

                    db.channels.update_one({
                        "_id": subChannelId
                    }, {
                        "$set": channels[subChannelId]
                    }, True)

                    logger.info("updated %s", subChannelId)
                except Exception, e:
                    logger.exception(e)

                # read sub level subscriptions
                subLevel = level + 1
                if readSubs == True:
                    readSubscriptions(subChannelId, subLevel)
    except Exception, e:
        logger.exception(e)


def readSubscriptionsPage(channelId, pageToken=None, level=1):

    url = "https://www.googleapis.com/youtube/v3/subscriptions?part=snippet&maxResults=50&channelId=" + \
        channelId + "&key=" + apiKeyProvider.apiKey()

    if pageToken != None:
        url += "&pageToken=" + pageToken

    # fetch subscriptions of channel
    r = requests.get(url)
    subs = r.json()

    # error? ignore!
    if r.status_code != 200:
        logger.warning("error %d %s %d", r.status_code, channelId, level)
        return None

    # loop channel items in result set
    for i in subs["items"]:

        if i["snippet"]["resourceId"]["kind"] != "youtube#channel":
            continue

        subChannelId = i["snippet"]["resourceId"]["channelId"]

        # store this channel
        upsertChannel(subChannelId, level, True)

    # is there a next page?
    if subs.has_key("nextPageToken"):
        return subs["nextPageToken"]
    else:
        return None


def readSubscriptions(channelId, level=1):

    # we reached the maximum level of recursion
    if level >= maxLevels:
        return None

    nextPage = None
    nextPageNew = None

    while True:
        nextPageNew = readSubscriptionsPage(channelId, nextPage, level)

        if nextPageNew == None:
            break

        nextPage = nextPageNew


def addAdditionalSubscriptions():

    adds = []

    for cc in db.additional.find({}):
        if not channels.has_key(cc["_id"]):
            adds.append(cc["_id"])

    for a in adds:

        try:
            logger.info("Additional channel %s will be added", a)

            # add this channel
            upsertChannel(a, 0, False, True)

            # check if channel is now available
            check_channel = db.channels.find_one({"_id": a})
            if check_channel != None:
                logger.info(
                    "Additional channel %s will be deleted now, it's added to the channels list", a)

                db.additional.delete_one({"_id": a})

        except Exception, e:
            logger.exception(e)


def checkAllChannelsForNewVideos():

    for channel in db.channels.find({}, projection=["_id"]):
        try:
            logger.info("Store videos via RSS feed for channel %s",
                        channel["_id"])

            maxPublishedAt = storeVideosFromRSSFeed(channel["_id"])

            # update videoCount and lastUploadAt on channel document
            db.channels.update_one({
                "_id": channel["_id"]
            }, {
                "$set": {
                    "videoCount": db.videos.count({"channel": channel["_id"]}),
                    "lastUploadAt": maxPublishedAt
                }
            })

        except Exception, e:
            logger.exception(e)


def storeVideosFromRSSFeed(channelId):
    headers = {
        "User-Agent": "Opera/9.80 (Windows NT 6.1; WOW64) Presto/2.12.388 Version/12.18"}

    url = "https://www.youtube.com/feeds/videos.xml?channel_id=" + channelId

    r = requests.get(url, headers=headers)

    if r.status_code != 200:
        logger.warning("request to %s failed with status code %d, body: %s",
                       url, r.status_code, r.content)
        return

    videoFeed = xmltodict.parse(r.content)

    if videoFeed is None or "feed" not in videoFeed:
        logger.warning("xml for url %s does not contain valid feed", url)
        return

    if "entry" not in videoFeed["feed"]:
        logger.warning(
            "xml for url %s does not contain any video entries", url)
        return

    maxPublishedAt = 0

    for feedItem in videoFeed["feed"]["entry"]:
        try:
            storedVideo = db.videos.find_one(
                {"_id": feedItem["yt:videoId"]}, projection=["_id", "updatedAt"])

            publishedAt = storeVideo(feedItem)

            if publishedAt > maxPublishedAt:
                maxPublishedAt = publishedAt

        except Exception, e:
            logger.exception(e)

    return maxPublishedAt


def storeVideo(feedItem):
    publishedAt = arrow.get(feedItem["published"]).timestamp
    updatedAt = arrow.get(feedItem["updated"]).timestamp

    vid = {
        "id": feedItem["yt:videoId"],
        "title": feedItem["title"],
        "description": feedItem["media:group"]["media:description"],
        "publishedAt": publishedAt,
        "updatedAt": updatedAt,
        "views": int(feedItem["media:group"]["media:community"]["media:statistics"]["@views"]),
        "channel": feedItem["yt:channelId"],
        "geoChecked": False,
        "tags": []
    }

    storeVideoWithStats(vid["channel"], vid)

    return publishedAt


def updateCurrentChannels():

    for cc in db.channels.find({}, projection=["_id"]):

        try:
            logger.info("Update existing channel %s with new data", cc["_id"])
            upsertChannel(cc["_id"], 0, False, True)
        except Exception, e:
            logger.exception(e)


def shouldReadSubscriptions():
    settings = db.settings.find_one({"_id": "lastSubscriberCrawl"})

    db.settings.update_one({"_id": "lastSubscriberCrawl"}, {
                           "$set": {"value": arrow.utcnow().timestamp}}, upsert=True)

    if settings is not None and "value" in settings:
        delta = math.fabs(int(settings["value"]) - arrow.utcnow().timestamp)
        result = delta >= ONE_DAY_IN_SECONDS

        logger.info(
            "read subscriptions? %s since time delta since last crawl is %d", result, delta)

        return result

    logger.info("no lastSubscriberCrawl yet, writing first one")

    return True


while True:

    if shouldReadSubscriptions():
        readSubscriptions(startChannelId, 1)

    checkAllChannelsForNewVideos()

    addAdditionalSubscriptions()

    updateCurrentChannels()

    logger.info("*** CYCLE COMPLETED ***")

    time.sleep(ONE_HOUR_IN_SECONDS / 4)
