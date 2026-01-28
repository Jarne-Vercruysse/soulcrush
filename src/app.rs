use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
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
#[cfg_attr(feature = "ssr", tracing::instrument(ret, err))]
async fn get_all_applications() -> Result<Vec<AllApplicationsResponse>, ServerFnError> {
    use sqlx::SqlitePool;

    let pool = expect_context::<SqlitePool>();

    let rows: Vec<ApplicationRow> = sqlx::query_as(
        r#"
        SELECT a.id, a.status, a.date,
               c.id as company_id, c.name, c.website, c.ceo, c.industry
        FROM applications a
        JOIN companies c ON a.company_id = c.id
        ORDER BY a.date DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Failed to fetch applications: {e}")))?;

    rows.into_iter().map(TryFrom::try_from).collect()
}

#[server]
#[cfg_attr(feature = "ssr", tracing::instrument(ret, err, fields(application_id = %id)))]
async fn delete_application(id: Uuid) -> Result<(), ServerFnError> {
    tracing::info!("deleting application");

    use sqlx::SqlitePool;
    let pool = expect_context::<SqlitePool>();

    sqlx::query("DELETE FROM applications WHERE id = ?")
        .bind(id.to_string())
        .execute(&pool)
        .await?;

    Ok(())
}

#[server]
#[cfg_attr(feature = "ssr", tracing::instrument(ret, err, skip(req), fields(company = %req.company.name)))]
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
    let delete = ServerAction::<DeleteApplication>::new();
    let create = ServerMultiAction::<CreateApplication>::new();

    provide_context(Resource::new(
        move || (delete.version().get(), create.version().get()),
        |_| get_all_applications(),
    ));
    provide_context(create);
    provide_context(delete);

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
                            <label for="req[company][name]">"Company Name"</label>
                            <input type="text" name="req[company][name]" required />
                        </div>
                        <div class="form-group">
                            <label for="req[company][website]">"Website"</label>
                            <input type="url" name="req[company][website]" required />
                        </div>
                    </div>

                    <div class="form-row">
                        <div class="form-group">
                            <label for="req[company][ceo]">"CEO"</label>
                            <input type="text" name="req[company][ceo]" required />
                        </div>
                        <div class="form-group">
                            <label for="req[company][industry]">"Industry"</label>
                            <input type="text" name="req[company][industry]" required />
                        </div>
                    </div>

                    <div class="form-row form-actions">
                        <div class="form-group">
                            <label for="req[status]">"Status"</label>
                            <select name="req[status]">
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

#[cfg(feature = "ssr")]
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

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct ApplicationRow {
    id: String,
    status: String,
    date: String,
    company_id: String,
    name: String,
    website: String,
    ceo: String,
    industry: String,
}

#[cfg(feature = "ssr")]
impl TryFrom<ApplicationRow> for AllApplicationsResponse {
    type Error = ServerFnError;

    fn try_from(r: ApplicationRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::parse_str(&r.id).map_err(|e| ServerFnError::new(e.to_string()))?,
            status: r
                .status
                .parse()
                .map_err(|e: String| ServerFnError::new(e))?,
            date: r.date,
            company: Company {
                id: Uuid::parse_str(&r.company_id)
                    .map_err(|e| ServerFnError::new(e.to_string()))?,
                name: r.name,
                website: r.website,
                ceo: r.ceo,
                industry: r.industry,
            },
        })
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
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

#[cfg(feature = "ssr")]
#[derive(Clone, PartialEq)]
struct Application {
    id: Uuid,
    company: Company,
    status: Status,
    date: OffsetDateTime,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
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

impl std::str::FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ToDo" => Ok(Status::ToDo),
            "Solicitated" => Ok(Status::Solicitated),
            "Pending" => Ok(Status::Pending),
            "Accepted" => Ok(Status::Accepted),
            "Rejected" => Ok(Status::Rejected),
            _ => Err(format!("Invalid status: {s}")),
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

#[cfg(feature = "ssr")]
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

#[cfg(feature = "ssr")]
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
