using System;

namespace Core.DTO.UseCaseRequests
{
    public class SelectAndStartChannelCrawlerRequest
    {
        public string ChannelId { get; set; }
        public int Level { get; set; } = 1;

        public SelectAndStartChannelCrawlerRequest(string channelId, int level)
        {
            ChannelId = channelId;
            Level = level;
        }
    }
}
