//! `common::jobs` (LEVELS.md §8.6): a job's steps, its end, every change
//! told, the ended ones kept up to `KEPT`.

use std::sync::{Arc, Mutex};

use super::*;

fn told() -> (Jobs, Arc<Mutex<Vec<Job>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let jobs = Jobs::new();
    let to = Arc::clone(&seen);
    jobs.set_emit(Box::new(move |job| to.lock().unwrap().push(job.clone())));
    (jobs, seen)
}

#[test]
fn a_job_goes_through_its_steps_and_ends() {
    let (jobs, seen) = told();
    let id = jobs.add(
        JobKind::VmPrepare,
        "Preparazione VM · shop".into(),
        Some("shop".into()),
        Vec::new(),
        JobState::Running,
        true,
    );
    jobs.update(&id, |j| j.begin_step(1, 3, "Strumenti".into()));
    jobs.update(&id, |j| {
        j.bytes = Some(JobBytes {
            done: 10,
            total: Some(100),
        });
    });
    jobs.update(&id, |j| j.begin_step(2, 3, "Database".into()));
    // The same step said again: its label changes, no new step.
    jobs.update(&id, |j| j.begin_step(2, 3, "Database del progetto".into()));
    let job = jobs.get(&id).unwrap();
    assert_eq!(job.steps.len(), 2);
    assert_eq!(job.steps[0].state, JobState::Done, "the one before ends");
    assert_eq!(job.step.as_ref().unwrap().label, "Database del progetto");
    assert_eq!(job.bytes, None, "a new step has no bytes yet");
    jobs.update(&id, |j| j.finish(JobState::Error, Some("sbx: no".into())));
    let job = jobs.get(&id).unwrap();
    assert_eq!(job.state, JobState::Error);
    assert_eq!(job.error.as_deref(), Some("sbx: no"));
    assert_eq!(job.steps[1].state, JobState::Error);
    assert_eq!(job.steps[1].error.as_deref(), Some("sbx: no"));
    // Told at every change: the add and five updates.
    assert_eq!(seen.lock().unwrap().len(), 6);
    assert_eq!(seen.lock().unwrap().last(), Some(&job));
}

#[test]
fn the_wire_names_are_the_interfaces() {
    let jobs = Jobs::new();
    let id = jobs.add(
        JobKind::VmPrepare,
        "t".into(),
        None,
        vec!["node".into()],
        JobState::Queued,
        false,
    );
    let json = serde_json::to_value(jobs.get(&id).unwrap()).unwrap();
    assert_eq!(json["kind"], "vm-prepare");
    assert_eq!(json["state"], "queued");
    assert_eq!(json["items"][0], "node");
    assert!(json.get("step").is_some() && json.get("bytes").is_some());
}

#[test]
fn only_the_last_ended_jobs_are_kept() {
    let jobs = Jobs::new();
    let running = jobs.add(
        JobKind::Install,
        "running".into(),
        None,
        Vec::new(),
        JobState::Running,
        false,
    );
    for i in 0..KEPT + 5 {
        let id = jobs.add(
            JobKind::Install,
            format!("{i}"),
            None,
            Vec::new(),
            JobState::Running,
            false,
        );
        jobs.update(&id, |j| j.finish(JobState::Done, None));
    }
    jobs.add(
        JobKind::Install,
        "last".into(),
        None,
        Vec::new(),
        JobState::Queued,
        false,
    );
    let list = jobs.list();
    assert_eq!(list.iter().filter(|j| j.state.is_over()).count(), KEPT);
    assert!(
        jobs.get(&running).is_some(),
        "a running job is never dropped"
    );
    assert_eq!(list.last().unwrap().title, "last");
}
