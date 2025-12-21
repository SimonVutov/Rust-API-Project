Rust and Javascript Notes Web App.\
Backend: `cd backend`, `cargo run`\
Frontend: `python3 -m http.server 3000`\
Go to `http://localhost:3000/`\
The backend is a Rust HTTP server ([std-only](https://doc.rust-lang.org/std/)) that serves a notes API with create/update/delete and persists notes to backend/data/note.json. The frontend is a dependency‑free static HTML/CSS/JS page that calls the API, served with a simple static file server on port 3000.\
The majority of this project is created using GPT-5.2-Codex for learning purposes.