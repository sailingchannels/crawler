using System;
using Hangfire;
using Hangfire.Mongo;
using Microsoft.Extensions.DependencyInjection;

namespace Infrastructure
{
    public static class AddJobQueueServices
    {
        public static void AddJobQueue(this IServiceCollection services)
        {
            // configure storage options for mongodb
            var storageOptions = new MongoStorageOptions
            {
                MigrationOptions = new MongoMigrationOptions
                {
                    Strategy = MongoMigrationStrategy.Migrate,
                    BackupStrategy = MongoBackupStrategy.Collections
                }
            };

            // read and prepare connection string
            string connString = "mongodb://localhost:27017";
            if (!string.IsNullOrWhiteSpace(Environment.GetEnvironmentVariable("MONGODB")))
            {
                connString = Environment.GetEnvironmentVariable("MONGODB");
            }

            services.AddHangfire(config =>
            {
                config.UseMongoStorage($"{connString}/sailing-channels-crawler", storageOptions);
            });

            // add the processing server as IHostedService
            services.AddHangfireServer();
        }
    }
}