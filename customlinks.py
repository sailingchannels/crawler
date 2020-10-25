import requests, json
from bs4 import BeautifulSoup

class CustomLinks:

	def getLinks(self, channelId):
		headers = {"User-Agent": "Opera/9.80 (Windows NT 6.1; WOW64) Presto/2.12.388 Version/12.18"}

		rd = requests.get("https://www.youtube.com/channel/" + channelId + "/about", headers=headers)
		
		if rd.status_code != 200:
			return None

		print rd.content

		soup = BeautifulSoup(rd.content, "html.parser")

		for link in soup.select("div#link-list-container"):
			print link

cl = CustomLinks()

cl.getLinks("UC-o2YdpshBrzlzEvOtu5Wlw")

