using System;
namespace Core.DTO.UseCaseResponses
{
    public class SelectAndStartChannelCrawlerResponse
    {
        public string Error { get; set; }

        public SelectAndStartChannelCrawlerResponse()
        {
        }

        public SelectAndStartChannelCrawlerResponse(string error)
        {
            Error = error;
        }
    }
}
