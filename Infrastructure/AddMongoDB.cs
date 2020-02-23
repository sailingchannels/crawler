using System;
using Core.Entities;
using Infrastructure.Mappings;
using Microsoft.Extensions.DependencyInjection;
using MongoDB.Driver;

namespace Infrastructure
{
    public static class MongoDB
    {
        public static void AddMongoDB(this IServiceCollection services)
        {
            // entity mappings
            services.AddMongoDBMappings();

            string connString = "mongodb://localhost:27017";
            if (!string.IsNullOrWhiteSpace(Environment.GetEnvironmentVariable("MONGODB")))
            {
                connString = Environment.GetEnvironmentVariable("MONGODB");
            }

            // database
            services.AddSingleton<IMongoClient>(f => new MongoClient());
            services.AddSingleton(f => f.GetRequiredService<IMongoClient>().GetDatabase("sailing-channels"));

            // collections
            services.AddSingleton(f => f.GetRequiredService<IMongoDatabase>().GetCollection<Channel>("channels"));
        }
    }
}