using System;
using System.Linq.Expressions;

namespace Core.Interfaces.Services
{
    public interface IJobQueueService
    {
        public string Enqueue<T>(Expression<Action<T>> methodCall);
    }
}
