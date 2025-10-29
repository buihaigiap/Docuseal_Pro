import { FieldType, Position } from '../../types';

export type EditorField = Omit<TemplateField, 'id'> & { id?: number; tempId: string };
export type FieldTool = 'cursor' | FieldType;

export interface DocumentEditorProps {
  template: Template;
  token: string | null;
}

export interface DocumentEditorRef {
  saveFields: () => Promise<void>;
}

export interface Template {
  id: number;
  fields?: TemplateField[];
  documents?: { url: string }[];
  file_url?: string;
}

export interface TemplateField {
  id: number;
  name: string;
  field_type: FieldType;
  required: boolean;
  display_order: number;
  position: Position;
  options?: any;
  partner?: string;
}

export interface NewTemplateField {
  name: string;
  field_type: FieldType;
  required: boolean;
  display_order: number;
  position: Position;
  options?: any;
  partner?: string;
}

export interface Field {
  id?: number;
  tempId: string;
  name: string;
  field_type: FieldType;
  required: boolean;
  display_order: number;
  position?: Position;
  options?: any;
  partner?: string;
}

export interface PartnerColorClasses {
  [key: string]: string;
}

export interface FieldToolItem {
  type: string;
  name: string;
  iconComponent: (className: string) => React.ReactElement;
}

export interface MobilePdfDimensions {
  width: number;
  height: number;
}

export interface DragState {
  x: number;
  y: number;
}

export interface InitialPosition extends DragState {
  width: number;
  height: number;
}

export interface ResizingColumn {
  tempId: string;
  index: number;
}