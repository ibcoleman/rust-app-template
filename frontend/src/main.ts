import {api} from "./api";
import type {Note} from "./types";

// DOM references
const appDiv = document.getElementById("app");
const greetingDiv = document.createElement("div");
greetingDiv.id = "greeting";

const notesSection = document.createElement("section");

const createForm = document.createElement("form");
createForm.innerHTML = `
  <h2>Create Note</h2>
  <input type="text" name="body" placeholder="Note content..." required />
  <button type="submit">Create</button>
`;

const notesList = document.createElement("ul");
notesList.id = "notes";

// Mount elements
if (appDiv) {
  appDiv.appendChild(greetingDiv);
  appDiv.appendChild(createForm);
  appDiv.appendChild(notesSection);
  notesSection.appendChild(document.createElement("h2")).textContent = "Notes";
  notesSection.appendChild(notesList);
}

// Load greeting on page load
async function loadGreeting(): Promise<void> {
  const result = await api.greet();

  result.match(
    (greeting) => {
      greetingDiv.textContent = greeting;
    },
    (error) => {
      greetingDiv.textContent = `Error: ${error.message}`;
    },
  );
}

// Load notes on page load
async function loadNotes(): Promise<void> {
  const result = await api.listNotes();

  result.match(
    (notes) => {
      renderNotes(notes);
    },
    (error) => {
      const li = document.createElement("li");
      li.textContent = `Error loading notes: ${error.message}`;
      notesList.appendChild(li);
    },
  );
}

function renderNotes(notes: Array<Note>): void {
  notesList.innerHTML = "";
  if (notes.length === 0) {
    const li = document.createElement("li");
    li.textContent = "No notes yet";
    notesList.appendChild(li);
    return;
  }

  for (const note of notes) {
    const li = document.createElement("li");
    li.textContent = `${note.body} (${note.created_at})`;
    notesList.appendChild(li);
  }
}

// Form submission
createForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  const form = e.target as HTMLFormElement;
  const formData = new FormData(form);
  const body = formData.get("body") as string;

  const result = await api.createNote(body);

  result.match(
    (_note) => {
      form.reset();
      loadNotes().catch((e) => console.error("Failed to reload notes:", e));
    },
    (error) => {
      alert(`Failed to create note: ${error.message}`);
    },
  );
});

// Initialize on load
loadGreeting().catch((e) => console.error("Failed to load greeting:", e));
loadNotes().catch((e) => console.error("Failed to load notes:", e));
