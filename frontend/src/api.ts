import {ResultAsync} from "neverthrow";
// @EXAMPLE-BLOCK-START notes
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
// @EXAMPLE-BLOCK-END notes

export const api = {
  greet: (name?: string) =>
    ResultAsync.fromPromise(
      fetch(
        name ? `/api/greet?name=${encodeURIComponent(name)}` : "/api/greet",
      ).then((r) => r.text()),
      (e) => ({message: String(e)}),
    ),
  // @EXAMPLE-BLOCK-START notes
  createNote: (body: string) =>
    jsonRequest<Note>("/api/notes", {
      method: "POST",
      headers: {"content-type": "application/json"},
      body: JSON.stringify({body}),
    }),
  listNotes: (limit = 20) => jsonRequest<Array<Note>>(`/api/notes?limit=${limit}`),
  getNote: (id: NoteId) => jsonRequest<Note>(`/api/notes/${id}`),
  // @EXAMPLE-BLOCK-END notes
};
