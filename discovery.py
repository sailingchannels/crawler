import requests
import config
import sys
import time
import math
import logging
import os
import arrow
from pymongo import MongoClient
from apikeyprovider import APIKeyProvider

apiKeyProvider = APIKeyProvider(config.apiKey())

ONE_HOUR_IN_SECONDS = 3600
ONE_DAY_IN_SECONDS = ONE_HOUR_IN_SECONDS * 24
ONE_MONTH_IN_SECONDS = ONE_HOUR_IN_SECONDS * 720

# logging
logger = logging.getLogger("sailing-channels-crawler")
logger.setLevel(logging.DEBUG)

ch = logging.StreamHandler()
formatter = logging.Formatter(
    '%(asctime)s - %(name)s - %(levelname)s - %(message)s')
ch.setFormatter(formatter)
logger.addHandler(ch)

print "environment", os.getenv("ENVIRONMENT")

# open mongodb connection
client = MongoClient(config.mongoDB())
db_name = "sailing-channels"
devMode = False

if len(sys.argv) != 2:
    db_name += "-dev"
    devMode = True
    logger.info("*** DEVELOPER MODE ***")

db = client[db_name]

sailingTerms = []
blacklist = []

# add sailing terms
for tt in db.sailingterms.find({}):
    sailingTerms.append(tt["_id"])

# fill blacklist
for bb in db.blacklist.find({}):
    blacklist.append(bb["_id"])


def readSubscriptionsPage(channelId, pageToken=None):

    url = "https://www.googleapis.com/youtube/v3/subscriptions?part=snippet&maxResults=50&channelId=" + \
        channelId + "&key=" + apiKeyProvider.apiKey()

    if pageToken != None:
        url += "&pageToken=" + pageToken

    # fetch subscriptions of channel
    r = requests.get(url)
    subs = r.json()

    # error? ignore!
    if r.status_code != 200:
        logger.warning("error %d %s", r.status_code, channelId)
        return None

    # loop channel items in result set
    for i in subs["items"]:

        if i["snippet"]["resourceId"]["kind"] != "youtube#channel":
            continue

        subChannelId = i["snippet"]["resourceId"]["channelId"]

        # check if channel exists in database
        channelCount = db.channels.count({"_id": subChannelId})
        additionalCount = db.additional.cound({"_id": subChannelId})
        channelIsNew = channelCount == 0 and additionalCount == 0

        # check if this channel has been seen as a non-sailing-channel before
        nonSailingChannelCount = db.nonsailingchannels.count(
            {"_id": subChannelId})

        channelIsNonSailingChannel = nonSailingChannelCount > 0

        if channelIsNew == True and channelIsNonSailingChannel == False:

            logger.info("New channel discovered: %s", subChannelId)

            db.additional.update_one({"_id": subChannelId}, {
                "$set": {
                    "_id": subChannelId,
                    "ignoreSailingTerm": False
                }
            }, True)

    # is there a next page?
    if subs.has_key("nextPageToken"):
        return subs["nextPageToken"]
    else:
        return None


def readSubscriptions(channelId):

    nextPage = None
    nextPageNew = None

    while True:
        nextPageNew = readSubscriptionsPage(channelId, nextPage)

        if nextPageNew == None:
            break

        nextPage = nextPageNew


def shouldReadSubscriptions():
    settings = db.settings.find_one({"_id": "lastSubscriberCrawl"})

    db.settings.update_one({"_id": "lastSubscriberCrawl"}, {
                           "$set": {"value": arrow.utcnow().timestamp}}, upsert=True)

    if settings is not None and "value" in settings:
        delta = math.fabs(int(settings["value"]) - arrow.utcnow().timestamp)
        result = delta >= ONE_DAY_IN_SECONDS

        logger.info(
            "read subscriptions? %s, since time delta since last crawl is %ds", result, delta)

        return result

    logger.info("no lastSubscriberCrawl yet, writing first one")

    return True


def getChannelIdsThatUploadedWithinLastThreeMonth():
    channelIds = []

    timestampThreeMonthAgo = arrow.utcnow().timestamp - (ONE_MONTH_IN_SECONDS * 3)

    for channel in db.channels.find({"lastUploadAt": {"$gte": timestampThreeMonthAgo}}, projection=["_id"]):
        channelIds.append(channel["_id"])

    return channelIds


while True:
    if shouldReadSubscriptions() or True:
        for channelId in getChannelIdsThatUploadedWithinLastThreeMonth():
            logger.info("Check subscriptions of channel %s", channelId)
            readSubscriptions(channelId)

    logger.info("*** CYCLE COMPLETED ***")

    time.sleep(ONE_HOUR_IN_SECONDS / 4)
