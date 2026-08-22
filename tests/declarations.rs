use rust_but_verify::{
    Access, Action, ActionId, Authorizer, Model, Scope, ScopedBy, Target, define_authorization,
};

define_authorization! {
    target KnowledgeBase = "knowledge_base" {
        action ReadArticle = "article.read";
        action CreateArticle = "article.create";
    };

    target Article = "article";
    target Wiki = "wiki";

    action EditArticle = "article.edit" for [Article, Wiki];
}

fn accepts_pair<A, S>(_action: A, _target: Target<S>)
where
    A: ScopedBy<S>,
    S: Scope,
{
}

#[test]
fn macro_declares_default_and_multiple_target_actions() {
    accepts_pair(ReadArticle, KnowledgeBase::ident("internal"));
    accepts_pair(EditArticle, Article::ident(1));
    accepts_pair(EditArticle, Wiki::ident(1));

    let mut policy = Model::builder();
    policy
        .grant("editor", ReadArticle)
        .grant("editor", EditArticle);
    let authorizer = Authorizer::new(policy.build().expect("policy should be valid"));

    assert!(
        authorizer
            .can(
                &Access::with_role("editor", KnowledgeBase::ident("internal")),
                ReadArticle,
                KnowledgeBase::ident("internal"),
            )
            .is_allowed()
    );
    assert_eq!(EditArticle.action_id(), ActionId::new("article.edit"));
}

mod ordinary_rust {
    use super::*;

    pub struct Document;

    impl Scope for Document {
        const KIND: &'static str = "document";
    }

    #[derive(Clone, Copy)]
    pub struct PublishDocument;

    impl Action for PublishDocument {
        fn action_id(&self) -> ActionId {
            ActionId::new("document.publish")
        }
    }

    impl ScopedBy<Document> for PublishDocument {}
}

#[test]
fn ordinary_rust_declarations_remain_supported() {
    accepts_pair(
        ordinary_rust::PublishDocument,
        ordinary_rust::Document::ident("handbook"),
    );
}
