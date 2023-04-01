mod entities {
    #[derive(Debug, Default)]
    pub struct Player {
        pub id: String,
        pub name: String,
        pub team_id: String,
    }

    #[derive(Debug, Default)]
    pub struct Team {
        pub id: String,
        pub name: String,
        pub missing_players: u64,
    }
}

mod repositories {
    pub mod postgres {
        use std::sync::Arc;

        pub struct Repo {
            pub pool: Arc<sqlx::PgPool>,
        }

        impl Repo {
            pub fn new(pool: Arc<sqlx::PgPool>) -> Self {
                Self { pool }
            }
        }

        pub mod player {
            pub mod commands {
                use crate::entities::Player;
                use crate::services::commands::player::create::PlayerCreateLambdaArgs;
                use crate::services::commands::Lambda;
                use crate::PlayerInput;

                #[async_trait::async_trait]
                impl crate::services::commands::RepoPlayer for crate::repositories::postgres::Repo {
                    async fn player_create(
                        &self,
                        input: &PlayerInput,
                        lambda: &Lambda<PlayerCreateLambdaArgs, Player>,
                    ) -> Result<Player, String> {
                        println!("input: {:?} - player_create postgres repo", input);

                        // create a transaction here because I can use it for other repository methods calls
                        let tx = self.pool.begin().await.unwrap();

                        // wait for lambda result
                        let player = lambda(PlayerCreateLambdaArgs {}).await?;

                        // insert player here with appropriate code for this repository

                        tx.commit().await.unwrap();

                        Ok(player)
                    }
                }
            }

            pub mod queries {
                use crate::entities::Player;

                #[async_trait::async_trait]
                impl crate::services::queries::RepoPlayer for crate::repositories::postgres::Repo {
                    async fn player_by_id(&self, id: &str) -> Result<Option<Player>, String> {
                        println!("id: {} - player_by_id postgres repo", id);

                        let obj = Player {
                            ..Default::default()
                        };

                        Ok(Some(obj))
                    }
                }
            }
        }

        pub mod team {
            pub mod queries {
                use crate::entities::Team;

                #[async_trait::async_trait]
                impl crate::services::queries::RepoTeam for crate::repositories::postgres::Repo {
                    async fn team_by_id(&self, id: &str) -> Result<Option<Team>, String> {
                        println!("id: {} - team_by_id postgres repo", id);

                        let obj = Team {
                            ..Default::default()
                        };

                        Ok(Some(obj))
                    }
                }
            }
        }
    }

    pub mod inmemory {
        use std::{collections::BTreeMap, sync::Arc};

        pub struct Repo {
            pub pool: Arc<BTreeMap<String, String>>,
        }

        impl Repo {
            pub fn new() -> Self {
                let pool = Arc::new(BTreeMap::new());

                Self { pool }
            }
        }

        pub mod player {
            pub mod commands {
                use crate::entities::Player;
                use crate::services::commands::player::create::PlayerCreateLambdaArgs;
                use crate::services::commands::Lambda;
                use crate::PlayerInput;

                #[async_trait::async_trait]
                impl crate::services::commands::RepoPlayer for crate::repositories::inmemory::Repo {
                    async fn player_create(
                        &self,
                        input: &PlayerInput,
                        lambda: &Lambda<PlayerCreateLambdaArgs, Player>,
                    ) -> Result<Player, String> {
                        println!("input: {:?} - player_create in_memory repo", input);

                        // create a transaction here because I can use it for other repository methods calls
                        // let mut tx = self.pool.begin().await?;

                        // wait for lambda result
                        let player = lambda(PlayerCreateLambdaArgs {}).await?;

                        // insert player here with appropriate code for this repository

                        // commit DB transaction here
                        // tx.commit().await?;

                        Ok(player)
                    }
                }
            }

            pub mod queries {
                pub mod player {
                    use crate::entities::Player;

                    #[async_trait::async_trait]
                    impl crate::services::queries::RepoPlayer for crate::repositories::inmemory::Repo {
                        async fn player_by_id(&self, id: &str) -> Result<Option<Player>, String> {
                            println!("id: {} - player_by_id in_memory repo", id);

                            let obj = Player {
                                ..Default::default()
                            };

                            Ok(Some(obj))
                        }
                    }
                }
            }
        }

        pub mod team {
            pub mod queries {
                use crate::entities::Team;

                #[async_trait::async_trait]
                impl crate::services::queries::RepoTeam for crate::repositories::inmemory::Repo {
                    async fn team_by_id(&self, id: &str) -> Result<Option<Team>, String> {
                        println!("id: {} - team_by_id in_memory repo", id);

                        let obj = Team {
                            ..Default::default()
                        };

                        Ok(Some(obj))
                    }
                }
            }
        }
    }
}

mod services {
    pub mod commands {
        use self::player::create::PlayerCreateLambdaArgs;
        use crate::entities::Player;
        use crate::services::commands::player::PlayerInput;
        use std::future::Future;
        use std::pin::Pin;

        pub mod player {
            #[derive(Debug, Default)]
            pub struct PlayerInput {
                pub name: String,
                pub team_id: String,
            }

            pub mod create {
                use crate::services::commands::player::PlayerInput;
                use crate::{entities::Player, Deps};
                use std::sync::Arc;

                pub struct Executor {
                    deps: Arc<Deps>,
                }

                impl Executor {
                    pub fn new(deps: Arc<Deps>) -> Self {
                        Self { deps }
                    }

                    pub async fn execute(&self, input: &PlayerInput) -> Result<Player, String> {
                        let res = self
                            .deps
                            .commands_repo
                            .player_create(input, &|_| {
                                let input = input;

                                Box::pin(async move {
                                    let obj = Player {
                                        id: "new_id".to_string(),
                                        name: input.name.to_owned(),
                                        team_id: input.team_id.to_owned(),
                                    };

                                    Ok(obj)
                                })
                            })
                            .await?;

                        Ok(res)
                    }
                }

                pub struct PlayerCreateLambdaArgs {}
            }
        }

        pub trait CommandsRepoTrait: Send + Sync + RepoPlayer {}

        impl<T: RepoPlayer> CommandsRepoTrait for T {}

        pub type Lambda<'a, ArgT, ResT> = dyn 'a
            + Fn(ArgT) -> Pin<Box<dyn Future<Output = Result<ResT, String>> + Send + 'a>>
            + Sync;

        #[async_trait::async_trait]
        pub trait RepoPlayer: Send + Sync {
            async fn player_create<'a>(
                &'a self,
                input: &PlayerInput,
                lambda: &Lambda<PlayerCreateLambdaArgs, Player>,
            ) -> Result<Player, String>;
        }
    }

    pub mod queries {
        use crate::entities::{Player, Team};

        pub mod player {
            pub mod find {
                use crate::{entities::Player, Deps};
                use std::sync::Arc;

                pub struct Executor {
                    deps: Arc<Deps>,
                }

                impl Executor {
                    pub fn new(deps: Arc<Deps>) -> Self {
                        Self { deps }
                    }

                    pub async fn execute(&self, id: &str) -> Result<Option<Player>, String> {
                        let res = self.deps.queries_repo.player_by_id(id).await?;

                        Ok(res)
                    }
                }
            }
        }

        pub mod team {
            pub mod find {
                use crate::{entities::Team, Deps};
                use std::sync::Arc;

                pub struct Executor {
                    deps: Arc<Deps>,
                }

                impl Executor {
                    pub fn new(deps: Arc<Deps>) -> Self {
                        Self { deps }
                    }

                    pub async fn execute(&self, id: &str) -> Result<Option<Team>, String> {
                        let res = self.deps.queries_repo.team_by_id(id).await?;

                        Ok(res)
                    }
                }
            }
        }

        pub trait QueriesRepoTrait: Send + Sync + RepoPlayer + RepoTeam {}

        impl<T: RepoPlayer + RepoTeam> QueriesRepoTrait for T {}

        #[async_trait::async_trait]
        pub trait RepoPlayer: Send + Sync {
            async fn player_by_id(&self, id: &str) -> Result<Option<Player>, String>;
        }

        #[async_trait::async_trait]
        pub trait RepoTeam: Send + Sync {
            async fn team_by_id(&self, id: &str) -> Result<Option<Team>, String>;
        }
    }
}

pub struct App {
    pub commands: Commands,
    pub queries: Queries,
}

pub struct Commands {
    pub player_create: services::commands::player::create::Executor,
}

pub struct Queries {
    pub player_by_id: services::queries::player::find::Executor,
    pub team_by_id: services::queries::team::find::Executor,
}

pub struct Deps {
    pub commands_repo: Arc<dyn services::commands::CommandsRepoTrait>,
    pub queries_repo: Arc<dyn services::queries::QueriesRepoTrait>,
}

use crate::services::commands::player::PlayerInput;
use std::sync::Arc;

trait RepoSuperTrait:
    services::commands::CommandsRepoTrait + services::queries::QueriesRepoTrait
{
}

impl<T: services::commands::CommandsRepoTrait + services::queries::QueriesRepoTrait> RepoSuperTrait
    for T
{
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let use_postgres = false;

    let db_repo: Arc<dyn RepoSuperTrait + 'static> = if use_postgres {
        let pg_pool = Arc::new(
            sqlx::PgPool::connect("postgres://postgres:postgres@localhost:5432/postgres")
                .await
                .unwrap(),
        );

        Arc::new(repositories::postgres::Repo::new(pg_pool))
    } else {
        Arc::new(repositories::inmemory::Repo::new())
    };

    let deps = Arc::new(Deps {
        commands_repo: db_repo.clone(),
        queries_repo: db_repo,
    });

    let app = App {
        commands: {
            Commands {
                player_create: services::commands::player::create::Executor::new(deps.clone()),
            }
        },

        queries: {
            Queries {
                player_by_id: services::queries::player::find::Executor::new(deps.clone()),
                team_by_id: services::queries::team::find::Executor::new(deps.clone()),
            }
        },
    };

    let new_player_input = PlayerInput {
        name: "Bob".to_string(),
        ..Default::default()
    };

    let new_player = app
        .commands
        .player_create
        .execute(&new_player_input)
        .await?;

    dbg!(&new_player);

    Ok(())
}
