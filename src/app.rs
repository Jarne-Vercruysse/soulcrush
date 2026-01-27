use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link href="https://unpkg.com/@csstools/normalize.css" rel="stylesheet" />
                <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=Orbitron:wght@500;600;700&display=swap" rel="stylesheet" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/soulcrush.css" />

        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router>
            <main class="container">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[server]
async fn get_all_applications() -> Result<Vec<AllApplicationsResponse>, ServerFnError> {
    unimplemented!();
}

#[server]
async fn delete_application(id: Uuid) -> Result<(), ServerFnError> {
    println!("Delete with id: {id}");

    Ok(())
}

#[server]
async fn create_application(req: CreateApplicationRequest) -> Result<(), ServerFnError> {
    use sqlx::SqlitePool;

    let pool = expect_context::<SqlitePool>();
    let company = Company::new(
        req.company.name,
        req.company.website,
        req.company.ceo,
        req.company.industry,
    );

    let application = Application::new(&company);

    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO companies (id, name, website, ceo, industry) VALUES (?, ?, ?, ?, ?)")
        .bind(company.id.to_string())
        .bind(company.name)
        .bind(company.website)
        .bind(company.ceo)
        .bind(company.industry)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO applications (id, company_id, status, date) VALUES (?, ?, ?, ?)")
        .bind(application.id.to_string())
        .bind(company.id.to_string())
        .bind(format!("{:?}", req.status))
        .bind(application.date.to_string())
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    provide_context(Resource::new(|| (), |_| get_all_applications()));
    provide_context(ServerAction::<DeleteApplication>::new());
    provide_context(ServerMultiAction::<CreateApplication>::new());

    view! {
        <h1>"Job Applications"</h1>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            <ApplicationList />
        </Suspense>
    }
}

#[component]
fn ApplicationList() -> impl IntoView {
    let applications =
        expect_context::<Resource<Result<Vec<AllApplicationsResponse>, ServerFnError>>>();

    view! {
        <CreateApplicationForm />
        <div class="application-list">
            <div class="list-header">
                <span>"Company"</span>
                <span>"Industry"</span>
                <span>"Link"</span>
                <span>"Status"</span>
                <span></span>
            </div>
            <Suspense fallback=|| ()>
                {move || Suspend::new(async move {
                    match applications.await {
                        Ok(data) => {
                            view! {
                                <For each=move || data.clone() key=|s| s.id let:application>
                                    <ApplicationCard application />
                                </For>
                            }
                                .into_any()
                        }
                        Err(_) => {
                            view! {
                                <div class="error">"Error loading applications"</div>
                            }
                                .into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn ApplicationCard(application: AllApplicationsResponse) -> impl IntoView {
    let delete_action = expect_context::<ServerAction<DeleteApplication>>();

    let id = application.id;
    let status = application.status;

    view! {
        <div class="application-card">
            <span class="card-company">{application.company.name.clone()}</span>
            <span class="card-industry">{application.company.industry.clone()}</span>
            <a href=application.company.website.clone() target="_blank" class="card-link">
                "Visit"
            </a>
            <button class=format!("status-badge {}", status.css_class())>
                {status.to_string()}
            </button>
            <ActionForm action=delete_action attr:class="card-delete">
                <input type="hidden" name="id" value=id.to_string() />
                <input class="btn-delete" type="submit" value="X" />
            </ActionForm>
        </div>
    }
}

#[component]
fn CreateApplicationForm() -> impl IntoView {
    let create_action = expect_context::<ServerMultiAction<CreateApplication>>();
    let is_open = RwSignal::new(false);

    view! {
        <div class="create-form-container">
            <button class="form-toggle" class:closed=move || !is_open.get() on:click=move |_| is_open.update(|v| *v = !*v)>
                <span class="toggle-icon" class:open=is_open>
                    "▶"
                </span>
                "New Application"
            </button>
            <Show when=move || is_open.get()>
                <MultiActionForm action=create_action attr:class="create-form">
                    <div class="form-row">
                        <div class="form-group">
                            <label for="name">"Company Name"</label>
                            <input type="text" name="name" required />
                        </div>
                        <div class="form-group">
                            <label for="website">"Website"</label>
                            <input type="url" name="website" required />
                        </div>
                    </div>

                    <div class="form-row">
                        <div class="form-group">
                            <label for="ceo">"CEO"</label>
                            <input type="text" name="ceo" required />
                        </div>
                        <div class="form-group">
                            <label for="industry">"Industry"</label>
                            <input type="text" name="industry" required />
                        </div>
                    </div>

                    <div class="form-row form-actions">
                        <div class="form-group">
                            <label for="status">"Status"</label>
                            <select name="status">
                                <option value="ToDo">"To Do"</option>
                                <option value="Solicitated">"Applied"</option>
                                <option value="Pending">"Pending"</option>
                                <option value="Accepted">"Accepted"</option>
                                <option value="Rejected">"Rejected"</option>
                            </select>
                        </div>
                        <button type="submit" class="btn-submit">
                            "Add Application"
                        </button>
                    </div>
                </MultiActionForm>
            </Show>
        </div>
    }
}

impl From<Application> for AllApplicationsResponse {
    fn from(s: Application) -> Self {
        Self {
            id: s.id,
            company: s.company,
            status: s.status,
            date: s.date.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
struct AllApplicationsResponse {
    id: Uuid,
    company: Company,
    status: Status,
    date: String,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
struct CreateApplicationRequest {
    company: CreateCompanyRequest,
    status: Status,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
struct CreateCompanyRequest {
    name: String,
    website: String,
    ceo: String,
    industry: String,
}

#[derive(Clone, PartialEq)]
struct Application {
    id: Uuid,
    company: Company,
    status: Status,
    date: OffsetDateTime,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
struct Company {
    id: Uuid,
    name: String,
    website: String,
    ceo: String,
    industry: String,
}

#[derive(Default, Clone, Copy, PartialEq, Deserialize, Serialize, Debug)]
enum Status {
    #[default]
    ToDo,
    Solicitated,
    Pending,
    Accepted,
    Rejected,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::ToDo => write!(f, "To Do"),
            Status::Solicitated => write!(f, "Solicitated"),
            Status::Pending => write!(f, "Pending"),
            Status::Accepted => write!(f, "Accepted"),
            Status::Rejected => write!(f, "Rejected"),
        }
    }
}

impl Status {
    fn next(self) -> Self {
        match self {
            Status::ToDo => Status::Solicitated,
            Status::Solicitated => Status::Pending,
            Status::Pending => Status::Accepted,
            Status::Accepted => Status::Rejected,
            Status::Rejected => Status::ToDo,
        }
    }

    fn css_class(&self) -> &'static str {
        match self {
            Status::ToDo => "status-todo",
            Status::Solicitated => "status-solicitated",
            Status::Pending => "status-pending",
            Status::Accepted => "status-accepted",
            Status::Rejected => "status-rejected",
        }
    }
}

impl Application {
    pub fn new(company: &Company) -> Self {
        Self {
            id: Uuid::new_v4(),
            company: company.clone(),
            status: Status::default(),
            date: OffsetDateTime::now_utc(),
        }
    }
}

impl Company {
    pub fn new(name: String, website: String, ceo: String, industry: String) -> Self {
        let id = Uuid::new_v4();

        Self {
            id,
            name,
            website,
            ceo,
            industry,
        }
    }
}
