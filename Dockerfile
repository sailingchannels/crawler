FROM ubuntu:16.04
MAINTAINER Thomas Brüggemann <mail@thomasbrueggemann.com>
LABEL Description="sailing-channels.com Crawler" Vendor="Sailing Channels" Version="1.13.9"

# INSTALL DEPENDENCIES
RUN apt-get update -y && apt-get install -y python-pip python-setuptools openssl python-dev libssl-dev cron
RUN pip install --upgrade pip

ADD . /srv
WORKDIR /srv

RUN pip install -r requirements.txt

# Add crontab file in the cron directory
ADD crontab /etc/cron.d/sailingchannels

# Give execution rights on the cron job
RUN chmod 0644 /etc/cron.d/sailingchannels

# Create the log file to be able to run tail
RUN touch /var/log/cron.log

# Run the command on container startup
CMD cron && tail -f /var/log/cron.log
