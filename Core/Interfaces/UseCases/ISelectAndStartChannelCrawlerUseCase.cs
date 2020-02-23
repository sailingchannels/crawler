using Core.DTO.UseCaseRequests;
using Core.DTO.UseCaseResponses;

namespace Core.Interfaces.UseCases
{
    public interface ISelectAndStartChannelCrawlerUseCase
        : IUseCase<
            SelectAndStartChannelCrawlerRequest,
            SelectAndStartChannelCrawlerResponse>
    {
    }
}
