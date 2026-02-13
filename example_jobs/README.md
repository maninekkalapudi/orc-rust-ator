# Example Jobs for DuckDB Integration

This directory contains example job configurations that demonstrate how to pull data from free APIs and load it into DuckDB.

## Available Examples

### 1. Weather Data (`weather_api_job.yaml`)
- **API**: [Open-Meteo](https://open-meteo.com/) - Free weather forecast API
- **Data**: Current temperature, humidity, wind speed, precipitation
- **Location**: New York City (configurable)
- **Schedule**: Every 6 hours
- **No API key required**

### 2. ISS Location Tracking (`iss_location_job.json`)
- **API**: [Open Notify](http://open-notify.org/) - ISS location tracker
- **Data**: Real-time latitude, longitude, timestamp of International Space Station
- **Schedule**: Every 5 minutes
- **No API key required**

### 3. Public Holidays (`holidays_job.json`)
- **API**: [Nager.Date](https://date.nager.at/) - Free public holidays API
- **Data**: Public holidays for various countries
- **Schedule**: Daily
- **No API key required**

## How to Use

### Option 1: Via API (Recommended)

```bash
# Create the job via API
curl -X POST http://localhost:8080/jobs \
  -H "Content-Type: application/json" \
  -d '{
    "job_name": "weather_data_ingestion",
    "description": "Fetch weather data from Open-Meteo API",
    "schedule": "0 */6 * * *",
    "is_active": true,
    "tasks": [
      {
        "extractor_config": {
          "type": "api",
          "url": "https://api.open-meteo.com/v1/forecast?latitude=40.7128&longitude=-74.0060&current=temperature_2m,relative_humidity_2m,wind_speed_10m,precipitation&timezone=America/New_York"
        },
        "loader_config": {
          "type": "warehouse",
          "target": "duckdb"
        }
      }
    ]
  }'
```

### Option 2: Manual Trigger

```bash
# Get the job ID from the create response, then trigger it
curl -X POST http://localhost:8080/jobs/{job_id}/run
```

### Option 3: Check Job Status

```bash
# List all jobs
curl http://localhost:8080/jobs

# Get specific job
curl http://localhost:8080/jobs/{job_id}

# Get job runs
curl http://localhost:8080/runs

# Get analytics for a job
curl http://localhost:8080/analytics/jobs/{job_id}
```

## Querying the Data in DuckDB

Once the job runs successfully, you can query the data:

```bash
# Connect to DuckDB container
podman exec -it orc-duckdb duckdb /data/warehouse.db

# Or via the analytics API
curl -X POST http://localhost:8080/analytics/query \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT * FROM job_results_{job_id} LIMIT 10"
  }'
```

## Free APIs You Can Use

Here are some excellent free APIs that don't require authentication:

1. **Weather**: https://open-meteo.com/
2. **ISS Location**: http://open-notify.org/
3. **Public Holidays**: https://date.nager.at/
4. **REST Countries**: https://restcountries.com/
5. **IP Geolocation**: https://ipapi.co/
6. **Exchange Rates**: https://exchangerate-api.com/
7. **Random User Data**: https://randomuser.me/
8. **NASA APOD**: https://api.nasa.gov/ (requires free API key)
9. **Dog Images**: https://dog.ceo/dog-api/
10. **GitHub Public Events**: https://api.github.com/events

## Customizing Jobs

You can customize the jobs by modifying:

- **URL parameters**: Change latitude/longitude for different locations
- **Schedule**: Use cron syntax (e.g., `0 0 * * *` for daily at midnight)
- **API endpoints**: Use different endpoints from the same API
- **Multiple tasks**: Chain multiple API calls in a single job

## Example: Custom Location Weather

```json
{
  "job_name": "london_weather",
  "description": "Weather data for London",
  "schedule": "0 */3 * * *",
  "is_active": true,
  "tasks": [
    {
      "extractor_config": {
        "type": "api",
        "url": "https://api.open-meteo.com/v1/forecast?latitude=51.5074&longitude=-0.1278&current=temperature_2m,relative_humidity_2m,wind_speed_10m&timezone=Europe/London"
      },
      "loader_config": {
        "type": "warehouse",
        "target": "duckdb"
      }
    }
  ]
}
```

## Troubleshooting

- **Job not running**: Check if `is_active` is set to `true`
- **API errors**: Verify the API endpoint is accessible
- **No data in DuckDB**: Check job run status via `/runs` endpoint
- **Schedule not working**: Ensure the scheduler service is running
