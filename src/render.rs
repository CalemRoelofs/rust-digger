use chrono::prelude::{DateTime, Utc};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::common::{get_owner_and_repo, percentage};
use crate::{collect_repos, Crate, CratesByOwner, Partials, Repo, User, PAGE_SIZE, VERSION};

pub fn render_list_crates_by_repo(repos: &Vec<Repo>) -> Result<(), Box<dyn Error>> {
    log::info!("render_list_crates_by_repo start");
    for repo in repos {
        // dbg!(&repo);
        render_list_page(
            &format!("_site/vcs/{}.html", repo.name),
            &format!("Crates in {}", repo.display),
            &repo.crates,
        )?;
    }
    log::info!("render_list_crates_by_repo end");
    Ok(())
}

pub fn render_list_of_repos(repos: &Vec<Repo>) {
    log::info!("render_list_of_repos start");
    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file("templates/repos.html")
        .unwrap();

    let filename = "_site/vcs/index.html";
    let utc: DateTime<Utc> = Utc::now();
    let globals = liquid::object!({
        "version": format!("{VERSION}"),
        "utc":     format!("{}", utc),
        "title":   "Repositories".to_string(),
        "repos":    repos,
    });
    let html = template.render(&globals).unwrap();
    let mut file = File::create(filename).unwrap();
    writeln!(&mut file, "{}", html).unwrap();
    log::info!("render_list_of_repos end");
}

pub fn read_file(filename: &str) -> String {
    let mut content = String::new();
    match File::open(filename) {
        Ok(mut file) => {
            file.read_to_string(&mut content).unwrap();
        }
        Err(error) => {
            println!("Error opening file {}: {}", filename, error);
        }
    }
    content
}

pub fn load_templates() -> Result<Partials, Box<dyn Error>> {
    // log::info!("load_templates");

    let mut partials = Partials::empty();
    let filename = "templates/incl/header.html";
    partials.add(filename, read_file(filename));
    let filename = "templates/incl/footer.html";
    partials.add(filename, read_file(filename));
    let filename = "templates/incl/navigation.html";
    partials.add(filename, read_file(filename));
    let filename = "templates/incl/list_crates.html";
    partials.add(filename, read_file(filename));

    Ok(partials)
}

pub fn render_static_pages() -> Result<(), Box<dyn Error>> {
    log::info!("render_static_pages start");

    let pages = vec![
        ("about", "About Rust Digger"),
        ("support", "Support Rust Digger"),
        ("training", "Training courses"),
    ];

    for page in pages {
        let partials = match load_templates() {
            Ok(partials) => partials,
            Err(error) => panic!("Error loading templates {}", error),
        };

        let utc: DateTime<Utc> = Utc::now();
        let globals = liquid::object!({
            "version": format!("{VERSION}"),
            "utc":     format!("{}", utc),
            "title":   page.1,
        });

        let template = liquid::ParserBuilder::with_stdlib()
            .partials(partials)
            .build()
            .unwrap()
            .parse_file(format!("templates/{}.html", page.0))
            .unwrap();
        let html = template.render(&globals).unwrap();

        let mut file = File::create(format!("_site/{}.html", page.0)).unwrap();
        writeln!(&mut file, "{}", html).unwrap();
    }
    log::info!("render_static_pages end");
    Ok(())
}

pub fn render_list_page(
    filename: &String,
    title: &String,
    crates: &Vec<Crate>,
) -> Result<(), Box<dyn Error>> {
    // log::info!("render {filename}");

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let page_size = if crates.len() > PAGE_SIZE {
        PAGE_SIZE
    } else {
        crates.len()
    };

    let utc: DateTime<Utc> = Utc::now();
    let globals = liquid::object!({
        "version": format!("{VERSION}"),
        "utc":     format!("{}", utc),
        "title":   title,
        "total":   crates.len(),
        "crates":  (&crates[0..page_size]).to_vec(),
    });

    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file("templates/crate_list_page.html")
        .unwrap();
    let html = template.render(&globals).unwrap();

    let mut file = File::create(filename).unwrap();
    writeln!(&mut file, "{}", html).unwrap();
    //match res {
    //    Ok(html) => writeln!(&mut file, "{}", html).unwrap(),
    //    Err(error) => println!("{}", error)
    //}
    Ok(())
}

pub fn render_news_pages() {
    log::info!("render_news_pages");
    let utc: DateTime<Utc> = Utc::now();

    let path = Path::new("templates/news");
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            let partials = match load_templates() {
                Ok(partials) => partials,
                Err(error) => panic!("Error loading templates {}", error),
            };
            if entry.path().extension().unwrap() != "html" {
                continue;
            }

            log::info!("news file: {:?}", entry.path());
            log::info!("{:?}", entry.path().strip_prefix("templates/"));
            let output_path = Path::new("_site")
                .join(entry.path().strip_prefix("templates/").unwrap().as_os_str());
            let template = liquid::ParserBuilder::with_stdlib()
                .partials(partials)
                .build()
                .unwrap()
                .parse_file(entry.path())
                .unwrap();

            let globals = liquid::object!({
                "version": format!("{VERSION}"),
                "utc":     format!("{}", utc),
            });
            let html = template.render(&globals).unwrap();
            //let filename = "_site/news.html";
            let mut file = File::create(output_path).unwrap();
            writeln!(&mut file, "{}", html).unwrap();
        }
    }

    //            },
    //            Err(error) => {
    //                println!("Error opening file {:?}: {}", file.as_os_str(), error);
    //            },
    //        }
    //    }
}

pub fn generate_crate_pages(crates: &Vec<Crate>) -> Result<(), Box<dyn Error>> {
    log::info!("generate_crate_pages start");
    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file("templates/crate.html")
        .unwrap();

    for krate in crates {
        let filename = format!("_site/crates/{}.html", krate.name);
        let utc: DateTime<Utc> = Utc::now();
        //log::info!("{:?}", krate);
        //std::process::exit(1);
        let globals = liquid::object!({
            "version": format!("{VERSION}"),
            "utc":     format!("{}", utc),
            "title":   &krate.name,
            "crate":   krate,
        });
        let html = template.render(&globals).unwrap();
        let mut file = File::create(filename).unwrap();
        writeln!(&mut file, "{}", html).unwrap();
    }
    log::info!("generate_crate_pages end");
    Ok(())
}

pub fn generate_user_pages(
    crates: &Vec<Crate>,
    users: Vec<User>,
    crates_by_owner: &CratesByOwner,
) -> Result<(), Box<dyn Error>> {
    log::info!("generate_user_pages start");

    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file("templates/user.html")
        .unwrap();

    let mut crate_by_id: HashMap<&str, &Crate> = HashMap::new();
    for krate in crates {
        crate_by_id.insert(&krate.id, krate);
    }
    //dbg!(&crate_by_id);
    //dbg!(&crate_by_id["81366"]);

    let mut users_with_crates: Vec<User> = users
        .into_iter()
        .map(|mut user| {
            let mut selected_crates: Vec<&Crate> = vec![];
            match crates_by_owner.get(&user.id) {
                Some(crate_ids) => {
                    //dbg!(crate_ids);
                    for crate_id in crate_ids {
                        //dbg!(&crate_id);
                        //dbg!(&crate_by_id[crate_id.as_str()]);
                        //dbg!(&crate_by_id.get(&crate_id.clone()));
                        selected_crates.push(&crate_by_id[crate_id.as_str()]);
                    }
                    user.count = selected_crates.len() as u16;
                    //users_with_crates.push(user);

                    selected_crates.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                    let filename =
                        format!("_site/users/{}.html", user.gh_login.to_ascii_lowercase());
                    let utc: DateTime<Utc> = Utc::now();
                    let globals = liquid::object!({
                        "version": format!("{VERSION}"),
                        "utc":     format!("{}", utc),
                        "title":   &user.name,
                        "user":    user,
                        "crates":  selected_crates,
                    });
                    let html = template.render(&globals).unwrap();
                    let mut file = File::create(filename).unwrap();
                    writeln!(&mut file, "{}", html).unwrap();
                }
                None => {
                    // We do not create a page for people who don't have crates.
                    //log::warn!("user {uid} does not have crates");
                }
            }
            user
        })
        .filter(|user| user.count > 0)
        .collect();

    users_with_crates.sort_by(|a, b| a.name.cmp(&b.name));

    generate_list_of_users(&users_with_crates);

    log::info!("generate_user_pages end");
    Ok(())
}

fn generate_list_of_users(users: &Vec<User>) {
    log::info!("generate_list_of_users start");
    // list all the users on the /users/ page
    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file("templates/users.html")
        .unwrap();

    let filename = "_site/users/index.html";
    let utc: DateTime<Utc> = Utc::now();
    let globals = liquid::object!({
        "version": format!("{VERSION}"),
        "utc":     format!("{}", utc),
        "title":   "Users".to_string(),
        "users":    users,
    });
    let html = template.render(&globals).unwrap();
    let mut file = File::create(filename).unwrap();
    writeln!(&mut file, "{}", html).unwrap();
    log::info!("generate_list_of_users end");
}

fn render_stats_page(
    crates: &Vec<Crate>,
    repos: &Vec<Repo>,
    home_page_but_no_repo: usize,
    no_homepage_no_repo_crates: usize,
    github_but_no_ci: usize,
    gitlab_but_no_ci: usize,
) {
    log::info!("render_stats_page");
    let partials = match load_templates() {
        Ok(partials) => partials,
        Err(error) => panic!("Error loading templates {}", error),
    };

    let template = liquid::ParserBuilder::with_stdlib()
        .partials(partials)
        .build()
        .unwrap()
        .parse_file("templates/stats.html")
        .unwrap();

    let filename = "_site/stats.html";
    let utc: DateTime<Utc> = Utc::now();
    let globals = liquid::object!({
        "version": format!("{VERSION}"),
        "utc":     format!("{}", utc),
        "title":   "Rust Digger Stats",
        //"user":    user,
        //"crate":   krate,
        "total": crates.len(),
        "repos": repos,
        "home_page_but_no_repo": home_page_but_no_repo,
        "home_page_but_no_repo_percentage":  percentage(home_page_but_no_repo, crates.len()),
        "no_homepage_no_repo_crates": no_homepage_no_repo_crates,
        "no_homepage_no_repo_crates_percentage": percentage(no_homepage_no_repo_crates, crates.len()),
        "github_but_no_ci": github_but_no_ci,
        "github_but_no_ci_percentage": percentage(github_but_no_ci, crates.len()),
        "gitlab_but_no_ci": gitlab_but_no_ci,
        "gitlab_but_no_ci_percentage": percentage(gitlab_but_no_ci, crates.len()),

    });
    let html = template.render(&globals).unwrap();
    let mut file = File::create(filename).unwrap();
    writeln!(&mut file, "{}", html).unwrap();
}

fn create_folders() {
    let _res = fs::create_dir_all("_site");
    let _res = fs::create_dir_all("_site/crates");
    let _res = fs::create_dir_all("_site/users");
    let _res = fs::create_dir_all("_site/news");
    let _res = fs::create_dir_all("_site/vcs");
}

pub fn generate_pages(crates: &Vec<Crate>) -> Result<(), Box<dyn Error>> {
    log::info!("generate_pages");

    create_folders();

    fs::copy("digger.js", "_site/digger.js")?;

    let repos = collect_repos(&crates);

    render_list_crates_by_repo(&repos)?;
    render_list_of_repos(&repos);

    let all_crates: Vec<Crate> = crates.into_iter().cloned().collect();
    render_list_page(
        &"_site/index.html".to_string(),
        &"Rust Digger".to_string(),
        &all_crates,
    )?;

    let github_but_no_ci = crates
        .into_iter()
        .filter(|w| on_github_but_no_ci(w))
        .cloned()
        .collect::<Vec<Crate>>();
    render_list_page(
        &"_site/github-but-no-ci.html".to_string(),
        &"On GitHub but has no CI".to_string(),
        &github_but_no_ci,
    )?;

    let gitlab_but_no_ci = crates
        .into_iter()
        .filter(|w| on_gitlab_but_no_ci(w))
        .cloned()
        .collect::<Vec<Crate>>();
    render_list_page(
        &"_site/gitlab-but-no-ci.html".to_string(),
        &"On GitLab but has no CI".to_string(),
        &gitlab_but_no_ci,
    )?;

    let home_page_but_no_repo = crates
        .into_iter()
        .filter(|w| has_homepage_no_repo(w))
        .cloned()
        .collect::<Vec<Crate>>();
    render_list_page(
        &"_site/has-homepage-but-no-repo.html".to_string(),
        &"Has homepage, but no repository".to_string(),
        &home_page_but_no_repo,
    )?;

    let no_homepage_no_repo_crates = crates
        .into_iter()
        .filter(|w| no_homepage_no_repo(w))
        .cloned()
        .collect::<Vec<Crate>>();
    render_list_page(
        &"_site/no-homepage-no-repo.html".to_string(),
        &"No repository, no homepage".to_string(),
        &no_homepage_no_repo_crates,
    )?;

    let crates_without_owner_name = crates
        .into_iter()
        .filter(|krate| krate.owner_name == "")
        .cloned()
        .collect::<Vec<Crate>>();
    render_list_page(
        &"_site/crates-without-owner-name.html".to_string(),
        &"Crates without owner name".to_string(),
        &crates_without_owner_name,
    )?;

    let crates_without_owner = crates
        .into_iter()
        .filter(|krate| krate.owner_name == "" && krate.owner_gh_login == "")
        .cloned()
        .collect::<Vec<Crate>>();

    render_list_page(
        &"_site/crates-without-owner.html".to_string(),
        &"Crates without owner".to_string(),
        &crates_without_owner,
    )?;

    //log::info!("repos: {:?}", repos);

    render_stats_page(
        crates,
        repos,
        home_page_but_no_repo.len(),
        no_homepage_no_repo_crates.len(),
        github_but_no_ci.len(),
        gitlab_but_no_ci.len(),
    );

    Ok(())
}

fn no_homepage_no_repo(w: &Crate) -> bool {
    w.homepage == "" && w.repository == ""
}

fn has_homepage_no_repo(w: &Crate) -> bool {
    w.homepage != "" && w.repository == ""
}

// fn has_repo(w: &Crate) -> bool {
//     w.repository != ""
// }
fn on_github_but_no_ci(krate: &Crate) -> bool {
    if krate.repository == "" {
        return false;
    }

    let (host, owner, _) = get_owner_and_repo(&krate.repository);
    if owner == "" {
        return false;
    }

    if host != "github" {
        return false;
    }

    if krate.details.has_github_action {
        return false;
    }

    true
}

fn on_gitlab_but_no_ci(krate: &Crate) -> bool {
    if krate.repository == "" {
        return false;
    }

    let (host, owner, _) = get_owner_and_repo(&krate.repository);
    if owner == "" {
        return false;
    }

    if host != "gitlab" {
        return false;
    }

    if krate.details.has_gitlab_pipeline {
        return false;
    }

    true
}
