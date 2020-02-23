using System;
using System.Collections.Generic;

namespace Core.Entities
{
    public class Channel : DisplayItem
    {
        public int Views { get; set; }
        public int PublishedAt { get; set; }
        public DateTime? LastCrawl { get; set; }
        public int Subscribers { get; set; }
        public int VideoCount { get; set; }
        public string Language { get; set; }
        public string Country { get; set; }
        public bool DetectedLanguage { get; set; }
        public int LastUploadAt { get; set; }
        public bool SubscribersHidden { get; set; }
        public ChannelPopularity Popularity { get; set; }
        public List<string> Keywords { get; set; } = new List<string>();
        public List<ChannelCustomLink> CustomLinks { get; set; } = new List<ChannelCustomLink>();
    }
}