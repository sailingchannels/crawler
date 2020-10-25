import requests
import json
import config
import calendar
import sys
import time
import math
import logging
from pymongo import MongoClient
from datetime import datetime, date, timedelta
import detectlanguage
from Queue import Queue
from twython import Twython
from apikeyprovider import APIKeyProvider
from colorformatter import ColorFormatter

apiKeyProvider = APIKeyProvider()

# logging
logger = logging.getLogger("sailing-channels-crawler")
logger.setLevel(logging.DEBUG)
ch = logging.StreamHandler()
formatter = logging.Formatter(
    '%(asctime)s - %(name)s - %(levelname)s - %(message)s')
ch.setFormatter(ColorFormatter())
logger.addHandler(ch)

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

# DELETE CHANNEL


def deleteChannel(channelId):
    db.channels.delete_one({"_id": channelId})
    db.videos.delete_many({"channel": channelId})
    db.views.delete_many({"_id.channel": channelId})
    db.visits.delete_many({"channel": channelId})
    db.subscribers.delete_many({"_id.channel": channelId})

# STORE VIDEO STATS


def storeVideoStats(channelId, vid):

    # fetch video statistics
    key = apiKeyProvider.apiKey()
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
        del dbVid["id"]

        try:
            # check if this video exists in database
            vid_exists = db.videos.count({"_id": dbVid["_id"]})

            # reasonable fresh video, post to twitter and facebook
            if vid_exists == 0 and math.fabs(int(dbVid["publishedAt"]) - time.mktime(datetime.utcnow().timetuple())) <= 15000:

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

        except:
            pass
    else:

        # remove non public videos
        db.videos.delete_one({
            "_id": dbVid["id"]
        })

# READ VIDEOS PAGE


def readVideosPage(channelId, pageToken=None):

    url = "https://www.googleapis.com/youtube/v3/search?part=snippet&channelId=" + \
        channelId + "&maxResults=50&regionCode=us&key=" + apiKeyProvider.apiKey()

    if pageToken != None:
        url += "&pageToken=" + pageToken

    # fetch videos of channel
    r = requests.get(url)
    vids = r.json()

    videos = []
    for v in vids["items"]:

        # ignore playlists
        if v["id"]["kind"] != "youtube#video":
            continue

        d = datetime.strptime(
            v["snippet"]["publishedAt"], "%Y-%m-%dT%H:%M:%SZ")

        vid = {
            "id": v["id"]["videoId"],
            "title": v["snippet"]["title"],
            "description": v["snippet"]["description"],
            "publishedAt": calendar.timegm(d.utctimetuple())
        }

        videos.append(vid)

        storeVideoStats(channelId, vid)

    # is there a next page?
    if vids.has_key("nextPageToken"):
        return vids["nextPageToken"], videos
    else:
        return None, videos

# READ VIDEOS


def readVideos(channelId):

    nextPage = None
    nextPageNew = None
    videos = []

    while True:
        nextPageNew, vids = readVideosPage(channelId, nextPage)

        # extend video list with new page ones
        videos.extend(vids)

        if nextPageNew == None:
            break

        nextPage = nextPageNew

    return videos

# READ STATISTICS


def readStatistics(channelId):

    r = requests.get("https://www.googleapis.com/youtube/v3/channels?part=statistics,snippet,brandingSettings&id=" +
                     channelId + "&key=" + apiKeyProvider.apiKey())
    stats = r.json()

    if stats is None or "items" not in stats:
        return None, None, None

    return stats["items"][0]["statistics"], stats["items"][0]["snippet"], stats["items"][0]["brandingSettings"]

# GET CHANNEL POPULARITY INDEX


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

# ADD SINGLE CHANNEL


def addSingleChannel(subChannelId, i, level, readSubs=True, ignoreSailingTerm=False):

    try:

        # store this channel
        if not channels.has_key(subChannelId):

            last_crawled = db.channels.find_one(
                {"_id": subChannelId}, projection=["lastCrawl"])
            if last_crawled:
                if (datetime.now() - last_crawled["lastCrawl"]).total_seconds() < THREE_HOURS_IN_SECONDS:
                    logger.info("skip %s lass crawl less than %d secs ago",
                                subChannelId, THREE_HOURS_IN_SECONDS)
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
                if (term in i["snippet"]["title"].lower() or term in i["snippet"]["description"].lower()):
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
                    "title": i["snippet"]["title"],
                    "description": i["snippet"]["description"],
                    "publishedAt": calendar.timegm(pd.utctimetuple()),
                    "thumbnail": i["snippet"]["thumbnails"]["default"]["url"],
                    "subscribers": int(stats["subscriberCount"]),
                    "views": int(stats["viewCount"]),
                    "subscribersHidden": bool(stats["hiddenSubscriberCount"]),
                    "lastCrawl": datetime.now()
                }

                # try to read custom links of channel
                # try:
                # 	rd = requests.get("https://sailing-channels.com/api/channel/get/" + subChannelId + "/customlinks")

                # 	if rd.status_code == 200:
                # 		customLinks = rd.json()
                # 		channels[subChannelId]["customLinks"] = customLinks
                # except Exception, e:
                # 	logger.exception(e)

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

                # read the videos
                channelVideos = readVideos(subChannelId)

                # video count
                channels[subChannelId]["videoCount"] = len(channelVideos)

                lotsOfText = channels[subChannelId]["description"] + " "

                # last upload at
                maxVideoAge = 0
                for vid in channelVideos:
                    lotsOfText += vid["description"] + " "
                    if vid["publishedAt"] > maxVideoAge:
                        maxVideoAge = vid["publishedAt"]

                channels[subChannelId]["lastUploadAt"] = maxVideoAge

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

# READ SUBSCRIPTIONS PAGE


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
        addSingleChannel(subChannelId, i, level, True)

    # is there a next page?
    if subs.has_key("nextPageToken"):
        return subs["nextPageToken"]
    else:
        return None

# READ SUBSCRIPTIONS


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

# ADDITIONAL SUBSCRIPTIONS


def addAdditionalSubscriptions():

    adds = []

    for cc in db.additional.find({}):
        if not channels.has_key(cc["_id"]):
            adds.append(cc["_id"])

    for a in adds:

        # get info of additional channel
        r = requests.get("https://www.googleapis.com/youtube/v3/channels?part=snippet&id=" +
                         a + "&key=" + apiKeyProvider.apiKey())
        result = r.json()

        try:
            logger.info("additional %s", a)

            # add this channel
            addSingleChannel(a, result["items"][0], 0, False, True)

            # check if channel is now available
            check_channel = db.channels.find_one({"_id": a})
            if check_channel != None:
                db.additional.delete_one({"_id": a})

        except Exception, e:
            logger.exception(e)

# READ CURRENT CHANNELS


def readCurrentChannels():

    for cc in db.channels.find({"lastCrawl": {"$lt": datetime.now() - timedelta(hours=1)}}, projection=["_id"], limit=10):

        try:
            # get info of additional channel
            r = requests.get("https://www.googleapis.com/youtube/v3/channels?part=snippet&id=" +
                             cc["_id"] + "&key=" + apiKeyProvider.apiKey())
            result = r.json()

            if result is not None and "items" in result and len(result["items"]) > 0:
                addSingleChannel(cc["_id"], result["items"][0], 0, False, True)
        except Exception, e:
            logger.exception(e)


# readCurrentChannels()
readSubscriptions(startChannelId, 1)
addAdditionalSubscriptions()
