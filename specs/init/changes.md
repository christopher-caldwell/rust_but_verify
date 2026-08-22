A few things standout right away as ergonomic painpoints. 

------------------------------------------------------------------------

```rust
impl Action for ReadArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "read")
    }
}

impl Action for CreateArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "create")
    }
}

impl Action for EditArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "edit")
    }
}

impl Action for DeleteArticle {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("article", "delete")
    }
}


....


impl ScopedBy<KnowledgeBase> for ReadArticle {}
impl ScopedBy<KnowledgeBase> for CreateArticle {}
impl ScopedBy<KnowledgeBase> for EditArticle {}
impl ScopedBy<KnowledgeBase> for DeleteArticle {}
```

This is going to get tedious real fast. I am unsure if a macro is the best way to handle this, or if this needs s second look. I know why we are using structs, but this could be a real real pain point. Imagine having 100s of entities, and needing to do this for all of them. No way. 

The policy builder looks great. Those grant('admin', Read) are perfect.


-------------------------------------------------------

```rust
Access::new([RoleAssignment::new("editor",ScopeKey::new("knowledge_base", "internal"))])
```

This is clunky. The idea makes sense, but it needs a better API. ScopeKey is not a descriptive variable name, imo.

-------------------------------------------------------

THe rest looks solid. I want to see options for the 2 issues listed above. Do not begin implementing fixes yet, I want to pick what we do. 