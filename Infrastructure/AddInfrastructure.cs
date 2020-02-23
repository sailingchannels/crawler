using System;
using Core.Interfaces.Services;
using Google.Apis.Services;
using Google.Apis.YouTube.v3;
using Infrastructure.Services;
using Microsoft.Extensions.DependencyInjection;

namespace Infrastructure
{
    public static class AddInfrastructureServices
    {
        public static void AddInfrastructure(this IServiceCollection services)
        {
            // youtube api
            services.AddSingleton<YouTubeService>(s => new YouTubeService(new BaseClientService.Initializer()
            {
                ApiKey = Environment.GetEnvironmentVariable("GOOGLE_API_KEY"),
                ApplicationName = "Sailing-Channels"
            }));

            // repositories
            //services.AddScoped<IExecutionRepository, ExecutionRepository>();

            // use cases
            //services.AddScoped<IMergeSortExecutionUseCase, MergeSortExecutionUseCase>();

            // inject access to a job queue
            services.AddJobQueue();

            // services
            services.AddScoped<IJobQueueService, JobQueueService>();

        }
    }
}