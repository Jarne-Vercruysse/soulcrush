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
async fn get_all_solicitaties() -> Result<Vec<AllSolicitatiesRequest>, ServerFnError> {
    let result = mock_data()
        .iter()
        .map(|sol| AllSolicitatiesRequest::from(sol.clone()))
        .collect();

    Ok(result)
}

#[server]
async fn delete_solicitatie(id: Uuid) -> Result<(), ServerFnError> {
    println!("Delete with id: {id}");

    Ok(())
}

#[server]
async fn create_application(req: CreateSolicitatieRequest) -> Result<(), ServerFnError> {
    println!("add solicitatie");

    Ok(())
}

#[cfg(feature = "ssr")]
fn mock_data() -> Vec<Solicitatie> {
    vec![
        Solicitatie::new(Company::new(
            "Acme Corp".into(),
            "https://acme.com".into(),
            "John Doe".into(),
            "Technology".into(),
        )),
        Solicitatie::new(Company::new(
            "TechStart".into(),
            "https://techstart.io".into(),
            "Jane Smith".into(),
            "Startup".into(),
        )),
        Solicitatie::new(Company::new(
            "BigBank".into(),
            "https://bigbank.com".into(),
            "Bob Wilson".into(),
            "Finance".into(),
        )),
        Solicitatie::new(Company::new(
            "CloudNine".into(),
            "https://cloudnine.dev".into(),
            "Alice Chen".into(),
            "Cloud Services".into(),
        )),
        Solicitatie::new(Company::new(
            "DataFlow".into(),
            "https://dataflow.io".into(),
            "Mike Johnson".into(),
            "Data Analytics".into(),
        )),
        Solicitatie::new(Company::new(
            "GreenEnergy".into(),
            "https://greenenergy.nl".into(),
            "Eva de Vries".into(),
            "Renewable Energy".into(),
        )),
        Solicitatie::new(Company::new(
            "HealthTech".into(),
            "https://healthtech.com".into(),
            "Dr. Sarah Lee".into(),
            "Healthcare".into(),
        )),
        Solicitatie::new(Company::new(
            "LogiSmart".into(),
            "https://logismart.eu".into(),
            "Peter Bakker".into(),
            "Logistics".into(),
        )),
        Solicitatie::new(Company::new(
            "MediaWave".into(),
            "https://mediawave.tv".into(),
            "Lisa Martinez".into(),
            "Media".into(),
        )),
        Solicitatie::new(Company::new(
            "SecureNet".into(),
            "https://securenet.io".into(),
            "Tom Anderson".into(),
            "Cybersecurity".into(),
        )),
    ]
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    provide_context(Resource::new(|| (), |_| get_all_solicitaties()));
    provide_context(ServerAction::<DeleteSolicitatie>::new());
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
    let solicitaties =
        expect_context::<Resource<Result<Vec<AllSolicitatiesRequest>, ServerFnError>>>();

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
                    match solicitaties.await {
                        Ok(data) => {
                            view! {
                                <For each=move || data.clone() key=|s| s.id let:solicitatie>
                                    <ApplicationCard solicitatie />
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
fn ApplicationCard(solicitatie: AllSolicitatiesRequest) -> impl IntoView {
    let delete_action = expect_context::<ServerAction<DeleteSolicitatie>>();

    let id = solicitatie.id;
    let status = solicitatie.status;

    view! {
        <div class="application-card">
            <span class="card-company">{solicitatie.company.name.clone()}</span>
            <span class="card-industry">{solicitatie.company.industry.clone()}</span>
            <a href=solicitatie.company.website.clone() target="_blank" class="card-link">
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

impl From<Solicitatie> for AllSolicitatiesRequest {
    fn from(s: Solicitatie) -> Self {
        Self {
            id: s.id,
            company: s.company,
            status: s.status,
            date: s.date.to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
struct AllSolicitatiesRequest {
    id: Uuid,
    company: Company,
    status: Status,
    date: String,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
struct CreateSolicitatieRequest {
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
struct Solicitatie {
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

impl Solicitatie {
    pub fn new(company: Company) -> Self {
        let id = Uuid::new_v4();
        let status = Status::default();
        let date = OffsetDateTime::now_utc();

        Self {
            id,
            company,
            status,
            date,
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
