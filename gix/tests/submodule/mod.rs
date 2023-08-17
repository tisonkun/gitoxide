pub fn repo(name: &str) -> crate::Result<gix::Repository> {
    use crate::util::named_subrepo_opts;
    named_subrepo_opts("make_submodules.sh", name, gix::open::Options::isolated())
}

mod open {
    use crate::submodule::repo;
    use gix::submodule;

    #[test]
    fn various() -> crate::Result {
        for (name, expected) in [
            (
                "with-submodules",
                &[
                    (
                        "m1",
                        gix::submodule::State {
                            repository_exists: true,
                            is_old_form: false,
                            worktree_checkout: true,
                            superproject_configuration: true,
                        },
                    ),
                    (
                        "dir/m1",
                        gix::submodule::State {
                            repository_exists: true,
                            is_old_form: false,
                            worktree_checkout: true,
                            superproject_configuration: true,
                        },
                    ),
                ] as &[_],
            ),
            (
                "with-submodules-after-clone",
                &[(
                    "m1",
                    gix::submodule::State {
                        repository_exists: false,
                        is_old_form: false,
                        worktree_checkout: false,
                        superproject_configuration: true,
                    },
                )],
            ),
        ] {
            let repo = repo(name)?;
            for (sm, (name, expected)) in repo.submodules()?.expect("modules present").zip(expected) {
                assert_eq!(sm.name(), name);
                let state = sm.state()?;
                assert_eq!(&state, expected);

                let sm_repo = sm.open()?;
                assert_eq!(sm_repo.is_some(), state.repository_exists);
                if let Some(sm_repo) = sm_repo {
                    assert_eq!(
                        sm_repo.kind(),
                        gix::repository::Kind::Submodule,
                        "Submodules are properly detected"
                    );
                    assert!(sm_repo.work_dir().is_some(), "the workdir is always configured");
                    let worktree = sm_repo
                        .worktree()
                        .expect("worktrees are always returned even if there seems to not be a checkout");
                    assert_eq!(
                        worktree.dot_git_exists(),
                        state.worktree_checkout,
                        "there is a way to check for indicators that a submodule worktree isn't checked out though"
                    )
                }
            }
            assert_eq!(
                repo.modules()?.expect("present").names().count(),
                expected.len(),
                "an expectation per submodule"
            );
        }
        Ok(())
    }

    #[test]
    fn old_form() -> crate::Result {
        for name in ["old-form-invalid-worktree-path", "old-form"] {
            let repo = repo(name)?;
            let sm = repo
                .submodules()?
                .expect("modules present")
                .next()
                .expect("one submodule");

            assert_ne!(
                sm.git_dir_try_old_form()?,
                sm.git_dir(),
                "compat git dir should be the worktree location"
            );
            let sm_repo = sm.open()?.expect("initialized");
            assert_eq!(
                sm_repo.kind(),
                gix::repository::Kind::WorkTree { is_linked: false },
                "old submodules aren't recognized as such because it would require reading a lot of additional data"
            );
            assert_eq!(
                sm.state()?,
                submodule::State {
                    repository_exists: true,
                    is_old_form: true,
                    worktree_checkout: true,
                    superproject_configuration: true,
                }
            );
        }
        Ok(())
    }
}
