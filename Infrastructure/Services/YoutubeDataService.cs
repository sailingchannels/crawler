using System;
using System.Threading.Tasks;
using Core.Interfaces.Services;
using Google.Apis.YouTube.v3;

namespace Infrastructure.Services
{
    public class YoutubeDataService : IYoutubeDataService
    {
        private readonly YouTubeService _youtube;

        public YoutubeDataService(YouTubeService youtube)
        {
            _youtube = youtube ?? throw new ArgumentNullException(nameof(youtube));
        }

        public async Task GetSubscriptions()
        {
            _youtube.Subscriptions.List("snippet")
        }
    }
}
