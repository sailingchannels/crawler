using System.Threading.Tasks;

namespace Core.Interfaces
{
    public interface IUseCase<in TUseCaseRequest, TUseCaseResponse>
    {
        Task<TUseCaseResponse> Handle(TUseCaseRequest message);
    }
}
