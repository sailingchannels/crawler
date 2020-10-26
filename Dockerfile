FROM ubuntu:16.04
MAINTAINER Thomas Brüggemann <mail@thomasbrueggemann.com>
LABEL Description="sailing-channels.com Crawler" Vendor="Sailing Channels" Version="2.0.3"

# INSTALL DEPENDENCIES
RUN apt-get update -y && apt-get install -y python-pip python-setuptools openssl python-dev libssl-dev cron
RUN pip install --upgrade pip

ADD . /srv
WORKDIR /srv

RUN pip install -r requirements.txt

# run the command on container startup
CMD python /srv/crawler.py /srv/