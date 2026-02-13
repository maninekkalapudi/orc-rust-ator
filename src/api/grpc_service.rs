use tonic::{Request, Response, Status};
use crate::state::db::Db;
use crate::orchestrator::job_manager::JobManager;
use uuid::Uuid;

pub mod proto {
    tonic::include_proto!("orc_rust_ator");
}

use proto::job_service_server::JobService;
use proto::{
    GetJobsRequest, GetJobsResponse,
    GetJobRequest, GetJobResponse,
    CreateJobRequest, CreateJobResponse,
    Job,
};

pub struct MyJobService {
    pub db: Db,
}

#[tonic::async_trait]
impl JobService for MyJobService {
    async fn get_jobs(
        &self,
        _request: Request<GetJobsRequest>,
    ) -> Result<Response<GetJobsResponse>, Status> {
        let jobs = self.db.get_all_job_definitions().await.map_err(|e| Status::internal(e.to_string()))?;
        
        let proto_jobs = jobs.into_iter().map(|j| Job {
            job_id: j.job_id.to_string(),
            job_name: j.job_name,
            description: j.description.unwrap_or_default(),
            schedule: j.schedule,
            is_active: j.is_active,
        }).collect();

        Ok(Response::new(GetJobsResponse { jobs: proto_jobs }))
    }

    async fn get_job(
        &self,
        request: Request<GetJobRequest>,
    ) -> Result<Response<GetJobResponse>, Status> {
        let req = request.into_inner();
        let job_uuid = Uuid::parse_str(&req.job_id).map_err(|_| Status::invalid_argument("Invalid UUID format"))?;
        let job_opt = self.db.get_job_definition(job_uuid).await.map_err(|e| Status::internal(e.to_string()))?;

        if let Some(j) = job_opt {
             let proto_job = Job {
                job_id: j.job_id.to_string(),
                job_name: j.job_name,
                description: j.description.unwrap_or_default(),
                schedule: j.schedule,
                is_active: j.is_active,
            };
            Ok(Response::new(GetJobResponse { job: Some(proto_job) }))
        } else {
            Err(Status::not_found("Job not found"))
        }
    }

    async fn create_job(
        &self,
        request: Request<CreateJobRequest>,
    ) -> Result<Response<CreateJobResponse>, Status> {
        let req = request.into_inner();
        
        let job_manager = JobManager::new(self.db.clone());
        let tasks = vec![]; 

        let job = job_manager.create_job(
            &req.job_name,
            if req.description.is_empty() { None } else { Some(&req.description) },
            &req.schedule,
            req.is_active,
            tasks,
        ).await.map_err(|e| Status::internal(e.to_string()))?;

        let proto_job = Job {
            job_id: job.job_id.to_string(),
            job_name: job.job_name,
            description: job.description.unwrap_or_default(),
            schedule: job.schedule,
            is_active: job.is_active,
        };

        Ok(Response::new(CreateJobResponse { job: Some(proto_job) }))
    }
}
