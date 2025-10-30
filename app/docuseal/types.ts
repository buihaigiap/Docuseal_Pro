export interface User {
  id: number;
  name: string;
  email: string;
  role: string;
  free_usage_count?: number;
  signature?: string;
  initials?: string;
}

export interface ApiResponse<T> {
  success: boolean;
  status_code: number;
  message: string;
  data: T;
  error?: string;
}

export interface AuthData {
  token: string;
  user: User;
}

export interface Template {
  user_name: string;
  id: number;
  name: string;
  file_url: string;
  documents?: { 
    url: string;
    filename?: string;
    content_type?: string;
    size?: number;
  }[];
  created_at: string;
  user_id: number;
  slug: string;
  updated_at: string;
  fields?: TemplateField[];
}

export interface Position {
  x: number;
  y: number;
  width: number;
  height: number;
  page: number;
}

export type FieldType = 'text' | 'signature' | 'initials' | 'date' | 'checkbox' | 'number' | 'radio' | 'multiple' | 'select' | 'cells' | 'image' | 'file';

export interface TemplateField {
  id: number;
  name: string;
  field_type: FieldType | string;
  required: boolean;
  position: Position;
  display_order?: number;
  options?: any;
  partner?: string;
}

export interface NewTemplateField {
  name: string;
  field_type: FieldType | string;
  required: boolean;
  position: Position;
  display_order?: number;
  options?: string[];
  partner?: string;
}

export interface Signature {
  field_id: number;
  field_name: string;
  signature_value: string;
}

export interface Submitter {
  id: number;
  name: string;
  email: string;
  status: 'pending' | 'signed' | 'completed';
  token: string;
  template_id: number;
  user_id: number;
  signed_at: string | null;
  created_at: string;
  updated_at: string;
  bulk_signatures?: Signature[];
  template?: Template;
}

export interface TemplateFullInfo {
  template: Template;
  submitters: Submitter[];
  total_submitters: number;
  signatures?: any[];
}

export interface NewSubmitter {
    name: string;
    email: string;
}

export interface SubmissionSignaturesResponse {
  template_info: {
    id: number;
    name: string;
    slug: string;
    user_id: number;
    document: {
      filename: string;
      content_type: string;
      size: number;
      url: string;
    };
  };
  bulk_signatures: {
    field_id: number;
    field_info: TemplateField;
    field_name: string;
    signature_value: string;
  }[];
}