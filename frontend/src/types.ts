// @EXAMPLE-FILE notes
// Frontend types for the `Note` example domain — deleted by
// `just clean-examples`.

import type {Tagged} from "type-fest";

export type NoteId = Tagged<string, "NoteId">;

export interface Note {
  id: NoteId;
  body: string;
  created_at: string;
}
