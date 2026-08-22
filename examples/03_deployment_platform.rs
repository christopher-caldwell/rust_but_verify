//! Example 3: a deployment platform with users, service accounts, and related resources.
//!
//! A project is the authorization scope for project-wide deployment creation.
//! An environment has its own scope for the narrower act of deploying to it.
//! The application establishes the `Environment -> Project` relationship
//! before selecting which scope to check.

use rust_but_verify::{
    Access, Action, ActionId, Authorizer, Model, RoleAssignment, Scope, ScopedBy, TargetKey,
};

#[derive(Clone, Copy)]
struct ViewProject;
#[derive(Clone, Copy)]
struct ConfigureProject;
#[derive(Clone, Copy)]
struct CreateDeployment;
#[derive(Clone, Copy)]
struct ViewDeployment;
#[derive(Clone, Copy)]
struct CancelDeployment;
#[derive(Clone, Copy)]
struct PromoteDeployment;
#[derive(Clone, Copy)]
struct DeployEnvironment;

// Each typed action exposes the runtime permission it represents.
impl Action for ViewProject {
    fn action_id(&self) -> ActionId {
        ActionId::new("project.view")
    }
}

impl Action for ConfigureProject {
    fn action_id(&self) -> ActionId {
        ActionId::new("project.configure")
    }
}

impl Action for CreateDeployment {
    fn action_id(&self) -> ActionId {
        ActionId::new("deployment.create")
    }
}

impl Action for ViewDeployment {
    fn action_id(&self) -> ActionId {
        ActionId::new("deployment.view")
    }
}

impl Action for CancelDeployment {
    fn action_id(&self) -> ActionId {
        ActionId::new("deployment.cancel")
    }
}

impl Action for PromoteDeployment {
    fn action_id(&self) -> ActionId {
        ActionId::new("deployment.promote")
    }
}

impl Action for DeployEnvironment {
    fn action_id(&self) -> ActionId {
        ActionId::new("environment.deploy")
    }
}

struct Project;
struct Environment;

impl Scope for Project {
    const KIND: &'static str = "project";
}

impl Scope for Environment {
    const KIND: &'static str = "environment";
}

impl ScopedBy<Project> for ViewProject {}
impl ScopedBy<Project> for ConfigureProject {}
impl ScopedBy<Project> for CreateDeployment {}
impl ScopedBy<Project> for ViewDeployment {}
impl ScopedBy<Project> for CancelDeployment {}
impl ScopedBy<Project> for PromoteDeployment {}
impl ScopedBy<Environment> for DeployEnvironment {}

struct ProjectRecord {
    slug: &'static str,
}

struct EnvironmentRecord<'a> {
    name: &'static str,
    project: &'a ProjectRecord,
}

impl EnvironmentRecord<'_> {
    fn scope_id(&self) -> String {
        format!("{}:{}", self.project.slug, self.name)
    }
}

struct User {
    name: &'static str,
    access: Access,
}

struct ServiceAccount {
    name: &'static str,
    access: Access,
}

/// The application has verified that `environment` belongs to the project it
/// contains, then checks the project-wide permission used for deployment creation.
fn may_create_deployment(
    authz: &Authorizer,
    access: &Access,
    environment: &EnvironmentRecord<'_>,
) -> bool {
    authz
        .can(
            access,
            CreateDeployment,
            Project::ident(environment.project.slug),
        )
        .is_allowed()
}

fn may_deploy_to_environment(
    authz: &Authorizer,
    access: &Access,
    environment: &EnvironmentRecord<'_>,
) -> bool {
    authz
        .can(
            access,
            DeployEnvironment,
            Environment::ident(environment.scope_id()),
        )
        .is_allowed()
}

fn main() -> Result<(), rust_but_verify::ModelError> {
    let mut policy = Model::builder();
    policy
        .grant("owner", ViewProject)
        .grant("owner", ConfigureProject)
        .grant("owner", CreateDeployment)
        .grant("owner", ViewDeployment)
        .grant("owner", CancelDeployment)
        .grant("owner", PromoteDeployment)
        .grant("developer", ViewProject)
        .grant("developer", CreateDeployment)
        .grant("developer", ViewDeployment)
        .grant("developer", CancelDeployment)
        .grant("deployer", CreateDeployment)
        .grant("deployer", ViewDeployment)
        .grant("deployer", DeployEnvironment);
    let authz = Authorizer::new(policy.build()?);

    let billing_api = ProjectRecord {
        slug: "billing-api",
    };
    let staging = EnvironmentRecord {
        name: "staging",
        project: &billing_api,
    };
    let production = EnvironmentRecord {
        name: "production",
        project: &billing_api,
    };

    let alice = User {
        name: "Alice",
        access: Access::new([RoleAssignment::new(
            "owner",
            TargetKey::new("project", "billing-api"),
        )]),
    };
    let bob = User {
        name: "Bob",
        access: Access::new([RoleAssignment::new(
            "developer",
            TargetKey::new("project", "billing-api"),
        )]),
    };
    let github_ci = ServiceAccount {
        name: "github-ci",
        access: Access::new([
            RoleAssignment::new("deployer", TargetKey::new("project", "billing-api")),
            RoleAssignment::new(
                "deployer",
                TargetKey::new("environment", staging.scope_id()),
            ),
        ]),
    };

    let bob_creates = may_create_deployment(&authz, &bob.access, &staging);
    println!(
        "{} creates a billing-api deployment: {bob_creates}",
        bob.name
    );

    let ci_creates = may_create_deployment(&authz, &github_ci.access, &staging);
    let ci_deploys_staging = may_deploy_to_environment(&authz, &github_ci.access, &staging);
    let ci_deploys_production = may_deploy_to_environment(&authz, &github_ci.access, &production);
    println!(
        "{} creates builds for billing-api: {ci_creates}",
        github_ci.name
    );
    println!(
        "{} deploys to staging: {ci_deploys_staging}",
        github_ci.name
    );
    println!(
        "{} deploys to production: {ci_deploys_production}",
        github_ci.name
    );

    let ci_configures = authz
        .can(
            &github_ci.access,
            ConfigureProject,
            Project::ident(billing_api.slug),
        )
        .is_allowed();
    let alice_configures = authz
        .can(
            &alice.access,
            ConfigureProject,
            Project::ident(billing_api.slug),
        )
        .is_allowed();
    println!("{} configures billing-api: {ci_configures}", github_ci.name);
    println!("{} configures billing-api: {alice_configures}", alice.name);

    assert!(bob_creates && ci_creates && ci_deploys_staging && alice_configures);
    assert!(!ci_deploys_production && !ci_configures);
    Ok(())
}
