import {ResultAsync} from "neverthrow";
import type {Note, NoteId} from "./types";

type ApiError = {message: string};

function jsonRequest<T>(
  path: string,
  init?: RequestInit,
): ResultAsync<T, ApiError> {
  return ResultAsync.fromPromise(
    fetch(path, init).then(async (r) => {
      if (!r.ok) throw new Error(`${r.status} ${r.statusText}`);
      return (await r.json()) as T;
    }),
    (e) => ({message: String(e)}),
  );
}

export const api = {
  greet: (name?: string) =>
    ResultAsync.fromPromise(
      fetch(
        name ? `/api/greet?name=${encodeURIComponent(name)}` : "/api/greet",
      ).then((r) => r.text()),
      (e) => ({message: String(e)}),
    ),
  createNote: (body: string) =>
    jsonRequest<Note>("/api/notes", {
      method: "POST",
      headers: {"content-type": "application/json"},
      body: JSON.stringify({body}),
    }),
  listNotes: (limit = 20) => jsonRequest<Array<Note>>(`/api/notes?limit=${limit}`),
  getNote: (id: NoteId) => jsonRequest<Note>(`/api/notes/${id}`),
};
