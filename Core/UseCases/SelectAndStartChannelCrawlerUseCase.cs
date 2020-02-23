using System;
using System.Threading.Tasks;
using Core.DTO.UseCaseRequests;
using Core.DTO.UseCaseResponses;
using Core.Interfaces.UseCases;

namespace Core.UseCases
{
    public class SelectAndStartChannelCrawlerUseCase
        : ISelectAndStartChannelCrawlerUseCase
    {
        public SelectAndStartChannelCrawlerUseCase()
        {
        }

        public async Task<SelectAndStartChannelCrawlerResponse> Handle(
            SelectAndStartChannelCrawlerRequest message
        )
        {
            // we reached the maximum level of recursion
            if (message.Level >= Constants.MAX_CHANNEL_DEPTH_LEVEL)
            {
                return new SelectAndStartChannelCrawlerResponse("max channel depth level reached");
            }

            string nextPage = null;

            do
            {

            }
            while (nextPage != null);
        }
    }
}
