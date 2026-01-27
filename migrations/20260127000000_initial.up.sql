CREATE TABLE companies (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    website TEXT NOT NULL,
    ceo TEXT NOT NULL,
    industry TEXT NOT NULL
);

CREATE TABLE applications (
    id TEXT PRIMARY KEY NOT NULL,
    company_id TEXT NOT NULL,
    status TEXT NOT NULL,
    date TEXT NOT NULL,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
