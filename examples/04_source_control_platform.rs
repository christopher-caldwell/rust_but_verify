//! Example 4: composing organization, team, repository, environment, and PR facts.
//!
//! This is intentionally a small application-side relationship layer. It
//! flattens verified memberships, parent relationships, and PR authorship
//! into the exact role assignments RustButVerify evaluates.

use rust_but_verify::{
    Access, Action, ActionId, Authorizer, Model, RoleAssignment, Scope, ScopedBy, TargetKey,
};

#[derive(Clone, Copy)]
struct ViewOrganization;
#[derive(Clone, Copy)]
struct ManageOrganizationMembers;
#[derive(Clone, Copy)]
struct Push;
#[derive(Clone, Copy)]
struct ConfigureRepository;
#[derive(Clone, Copy)]
struct ManageCollaborators;
#[derive(Clone, Copy)]
struct DeployEnvironment;
#[derive(Clone, Copy)]
struct EditPullRequest;
#[derive(Clone, Copy)]
struct ClosePullRequest;
#[derive(Clone, Copy)]
struct RequestReview;
#[derive(Clone, Copy)]
struct MergePullRequest;

// Explicit action declarations make every policy permission visible without
// asking the reader to expand a macro mentally.
impl Action for ViewOrganization {
    fn action_id(&self) -> ActionId {
        ActionId::new("organization.view")
    }
}

impl Action for ManageOrganizationMembers {
    fn action_id(&self) -> ActionId {
        ActionId::new("organization.manage_members")
    }
}

impl Action for Push {
    fn action_id(&self) -> ActionId {
        ActionId::new("repository.push")
    }
}

impl Action for ConfigureRepository {
    fn action_id(&self) -> ActionId {
        ActionId::new("repository.configure")
    }
}

impl Action for ManageCollaborators {
    fn action_id(&self) -> ActionId {
        ActionId::new("repository.manage_collaborators")
    }
}

impl Action for DeployEnvironment {
    fn action_id(&self) -> ActionId {
        ActionId::new("environment.deploy")
    }
}

impl Action for EditPullRequest {
    fn action_id(&self) -> ActionId {
        ActionId::new("pull_request.edit")
    }
}

impl Action for ClosePullRequest {
    fn action_id(&self) -> ActionId {
        ActionId::new("pull_request.close")
    }
}

impl Action for RequestReview {
    fn action_id(&self) -> ActionId {
        ActionId::new("pull_request.request_review")
    }
}

impl Action for MergePullRequest {
    fn action_id(&self) -> ActionId {
        ActionId::new("pull_request.merge")
    }
}

struct Organization;
struct Repository;
struct Environment;
struct PullRequest;

impl Scope for Organization {
    const KIND: &'static str = "organization";
}
impl Scope for Repository {
    const KIND: &'static str = "repository";
}
impl Scope for Environment {
    const KIND: &'static str = "environment";
}
impl Scope for PullRequest {
    const KIND: &'static str = "pull_request";
}

impl ScopedBy<Organization> for ViewOrganization {}
impl ScopedBy<Organization> for ManageOrganizationMembers {}
impl ScopedBy<Repository> for Push {}
impl ScopedBy<Repository> for ConfigureRepository {}
impl ScopedBy<Repository> for ManageCollaborators {}
impl ScopedBy<Repository> for MergePullRequest {}
impl ScopedBy<Environment> for DeployEnvironment {}
impl ScopedBy<PullRequest> for EditPullRequest {}
impl ScopedBy<PullRequest> for ClosePullRequest {}
impl ScopedBy<PullRequest> for RequestReview {}

struct OrganizationRecord {
    slug: &'static str,
}

struct Team<'a> {
    slug: &'static str,
    organization: &'a OrganizationRecord,
}

struct RepositoryRecord<'a> {
    slug: &'static str,
    organization: &'a OrganizationRecord,
}

struct EnvironmentRecord<'a> {
    name: &'static str,
    repository: &'a RepositoryRecord<'a>,
}

struct PullRequestRecord<'a> {
    number: u64,
    repository: &'a RepositoryRecord<'a>,
    author: &'static str,
}

impl RepositoryRecord<'_> {
    fn scope_id(&self) -> String {
        format!("{}:{}", self.organization.slug, self.slug)
    }
}

impl EnvironmentRecord<'_> {
    fn scope_id(&self) -> String {
        format!("{}:{}", self.repository.scope_id(), self.name)
    }
}

impl PullRequestRecord<'_> {
    fn scope_id(&self) -> String {
        format!("{}#{}", self.repository.scope_id(), self.number)
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

/// The application resolves Backend team membership and the team's grants
/// before giving Alice these effective role assignments. The library does not
/// itself follow `User -> Team -> Repository` edges.
fn backend_member_access(
    team: &Team<'_>,
    repository: &RepositoryRecord<'_>,
    staging: &EnvironmentRecord<'_>,
) -> Access {
    assert_eq!(team.organization.slug, repository.organization.slug);
    Access::new([
        RoleAssignment::new(
            "organization_member",
            TargetKey::new("organization", team.organization.slug),
        ),
        RoleAssignment::new(
            "repository_writer",
            TargetKey::new("repository", repository.scope_id()),
        ),
        RoleAssignment::new(
            "staging_deployer",
            TargetKey::new("environment", staging.scope_id()),
        ),
    ])
}

/// The caller has loaded the PR and checked its author before granting an
/// author-only role at the PR's exact scope.
fn contributor_access(author: &str, pull_request: &PullRequestRecord<'_>) -> Access {
    let mut access = Access::new([RoleAssignment::new(
        "repository_contributor",
        TargetKey::new("repository", pull_request.repository.scope_id()),
    )]);
    if author == pull_request.author {
        access.push(RoleAssignment::new(
            "pull_request_author",
            TargetKey::new("pull_request", pull_request.scope_id()),
        ));
    }
    access
}

/// A PR belongs to a repository, so maintainers merge by checking the parent
/// repository scope. Establishing that parent relationship is application work.
fn may_merge_pull_request(
    authz: &Authorizer,
    access: &Access,
    pull_request: &PullRequestRecord<'_>,
) -> bool {
    authz
        .can(
            access,
            MergePullRequest,
            Repository::ident(pull_request.repository.scope_id()),
        )
        .is_allowed()
}

fn main() -> Result<(), rust_but_verify::ModelError> {
    let mut policy = Model::builder();
    policy
        .grant("organization_member", ViewOrganization)
        .grant("repository_writer", Push)
        .grant("staging_deployer", DeployEnvironment)
        .grant("repository_admin", Push)
        .grant("repository_admin", ConfigureRepository)
        .grant("repository_admin", ManageCollaborators)
        .grant("repository_admin", MergePullRequest)
        .grant("repository_contributor", Push)
        .grant("pull_request_author", EditPullRequest)
        .grant("pull_request_author", ClosePullRequest)
        .grant("pull_request_author", RequestReview)
        .grant("repository_maintainer", MergePullRequest)
        .grant("production_deployer", DeployEnvironment);
    let authz = Authorizer::new(policy.build()?);

    let acme = OrganizationRecord { slug: "acme" };
    let backend = Team {
        slug: "backend",
        organization: &acme,
    };
    let payments_api = RepositoryRecord {
        slug: "payments-api",
        organization: &acme,
    };
    let staging = EnvironmentRecord {
        name: "staging",
        repository: &payments_api,
    };
    let production = EnvironmentRecord {
        name: "production",
        repository: &payments_api,
    };
    let pr_42 = PullRequestRecord {
        number: 42,
        repository: &payments_api,
        author: "Carol",
    };

    let alice = User {
        name: "Alice",
        access: backend_member_access(&backend, &payments_api, &staging),
    };
    println!("{} is a member of the {} team", alice.name, backend.slug);
    let bob = User {
        name: "Bob",
        access: Access::new([RoleAssignment::new(
            "repository_admin",
            TargetKey::new("repository", payments_api.scope_id()),
        )]),
    };
    let carol = User {
        name: "Carol",
        access: contributor_access("Carol", &pr_42),
    };
    let dana = User {
        name: "Dana",
        access: Access::new([RoleAssignment::new(
            "repository_maintainer",
            TargetKey::new("repository", payments_api.scope_id()),
        )]),
    };
    let deploy_production = ServiceAccount {
        name: "deploy-production",
        access: Access::new([RoleAssignment::new(
            "production_deployer",
            TargetKey::new("environment", production.scope_id()),
        )]),
    };

    let alice_pushes = authz
        .can(
            &alice.access,
            Push,
            Repository::ident(payments_api.scope_id()),
        )
        .is_allowed();
    let alice_staging = authz
        .can(
            &alice.access,
            DeployEnvironment,
            Environment::ident(staging.scope_id()),
        )
        .is_allowed();
    let alice_production = authz
        .can(
            &alice.access,
            DeployEnvironment,
            Environment::ident(production.scope_id()),
        )
        .is_allowed();
    println!("{} pushes through team access: {alice_pushes}", alice.name);
    println!("{} deploys to staging: {alice_staging}", alice.name);
    println!("{} deploys to production: {alice_production}", alice.name);

    let bob_configures = authz
        .can(
            &bob.access,
            ConfigureRepository,
            Repository::ident(payments_api.scope_id()),
        )
        .is_allowed();
    let bob_pushes = authz
        .can(
            &bob.access,
            Push,
            Repository::ident(payments_api.scope_id()),
        )
        .is_allowed();
    let bob_manages = authz
        .can(
            &bob.access,
            ManageCollaborators,
            Repository::ident(payments_api.scope_id()),
        )
        .is_allowed();
    let bob_merges = may_merge_pull_request(&authz, &bob.access, &pr_42);
    let bob_production = authz
        .can(
            &bob.access,
            DeployEnvironment,
            Environment::ident(production.scope_id()),
        )
        .is_allowed();
    println!("{} configures the repository: {bob_configures}", bob.name);
    println!("{} pushes to the repository: {bob_pushes}", bob.name);
    println!("{} manages collaborators: {bob_manages}", bob.name);
    println!("{} merges PR #42: {bob_merges}", bob.name);
    println!("{} deploys to production: {bob_production}", bob.name);

    let carol_edits = authz
        .can(
            &carol.access,
            EditPullRequest,
            PullRequest::ident(pr_42.scope_id()),
        )
        .is_allowed();
    let carol_closes = authz
        .can(
            &carol.access,
            ClosePullRequest,
            PullRequest::ident(pr_42.scope_id()),
        )
        .is_allowed();
    let carol_requests_review = authz
        .can(
            &carol.access,
            RequestReview,
            PullRequest::ident(pr_42.scope_id()),
        )
        .is_allowed();
    let carol_merges = may_merge_pull_request(&authz, &carol.access, &pr_42);
    let dana_merges = may_merge_pull_request(&authz, &dana.access, &pr_42);
    println!("{} edits PR #42: {carol_edits}", carol.name);
    println!("{} closes PR #42: {carol_closes}", carol.name);
    println!(
        "{} requests review on PR #42: {carol_requests_review}",
        carol.name
    );
    println!("{} merges PR #42: {carol_merges}", carol.name);
    println!("{} merges PR #42 as a maintainer: {dana_merges}", dana.name);

    let service_deploys = authz
        .can(
            &deploy_production.access,
            DeployEnvironment,
            Environment::ident(production.scope_id()),
        )
        .is_allowed();
    let service_configures = authz
        .can(
            &deploy_production.access,
            ConfigureRepository,
            Repository::ident(payments_api.scope_id()),
        )
        .is_allowed();
    let service_manages_members = authz
        .can(
            &deploy_production.access,
            ManageOrganizationMembers,
            Organization::ident(acme.slug),
        )
        .is_allowed();
    println!(
        "{} deploys to production: {service_deploys}",
        deploy_production.name
    );
    println!(
        "{} configures the repository: {service_configures}",
        deploy_production.name
    );
    println!(
        "{} manages organization membership: {service_manages_members}",
        deploy_production.name
    );

    assert!(
        alice_pushes && alice_staging && bob_configures && bob_pushes && bob_manages && bob_merges
    );
    assert!(carol_edits && carol_closes && carol_requests_review && dana_merges && service_deploys);
    assert!(!alice_production && !bob_production && !carol_merges);
    assert!(!service_configures && !service_manages_members);
    Ok(())
}
