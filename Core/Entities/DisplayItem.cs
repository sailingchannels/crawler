namespace Core.Entities
{
    public abstract class DisplayItem
    {
        public string ID { get; set; }
        public string Title { get; set; }
        public string Description { get; set; }
        public string Thumbnail { get; set; }
    }
}
