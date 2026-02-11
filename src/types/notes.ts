export type NoteCategory = "key_point" | "decision" | "action_item" | "risk";

export interface KeyPoint {
  topic: string;
  summary: string;
  timestamp: string;
}

export interface Decision {
  decision: string;
  rationale?: string;
  timestamp: string;
}

export interface ActionItem {
  task: string;
  owner?: string;
  deadline?: string;
  priority?: string;
}

export interface Risk {
  risk: string;
  impact?: string;
  mitigation?: string;
  timestamp: string;
}

export interface IncrementalNotesResponse {
  key_points: KeyPoint[];
  decisions: Decision[];
  action_items: ActionItem[];
  risks: Risk[];
}

export interface NoteRecord {
  id: number;
  meeting_id: number;
  note_type: NoteCategory;
  content: string; // JSON string of the bullet
  created_at: string;
}

export interface NotesUpdatedPayload {
  meeting_id: number;
  new_notes: IncrementalNotesResponse;
  total_count: number;
  inserted_ids: number[];
}
