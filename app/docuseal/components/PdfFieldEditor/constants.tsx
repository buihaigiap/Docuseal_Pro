import React from 'react';
import { FieldType } from '../../types';
import { FieldTool } from './types';
import { MousePointer, Type, PenTool, Hash, User, Calendar, CheckSquare, Circle, List, ChevronDown, Table, ImageIcon, File } from 'lucide-react';

export const fieldTools: { name: string; type: FieldTool; iconComponent: (className: string) => React.ReactElement }[] = [
  { name: 'Cursor', type: 'cursor', iconComponent: (className) => <MousePointer className={className} /> },
  { name: 'Text', type: 'text', iconComponent: (className) => <Type className={className} /> },
  { name: 'Signature', type: 'signature', iconComponent: (className) => <PenTool className={className} /> },
  { name: 'Number', type: 'number', iconComponent: (className) => <Hash className={className} /> },
  { name: 'Initials', type: 'initials', iconComponent: (className) => <User className={className} /> },
  { name: 'Date', type: 'date', iconComponent: (className) => <Calendar className={className} /> },
  { name: 'Checkbox', type: 'checkbox', iconComponent: (className) => <CheckSquare className={className} /> },
  { name: 'Radio', type: 'radio', iconComponent: (className) => <Circle className={className} /> },
  { name: 'Multiple', type: 'multiple', iconComponent: (className) => <List className={className} /> },
  { name: 'Select', type: 'select' as FieldType, iconComponent: (className) => <ChevronDown className={className} /> },
  { name: 'Cells', type: 'cells', iconComponent: (className) => <Table className={className} /> },
  { name: 'Image', type: 'image', iconComponent: (className) => <ImageIcon className={className} /> },
  { name: 'File', type: 'file', iconComponent: (className) => <File className={className} /> },
];

export const partnerColorClasses = [
  'bg-blue-500 bg-opacity-40 border-blue-400',
  'bg-green-500 bg-opacity-40 border-green-400',
  'bg-purple-500 bg-opacity-40 border-purple-400',
  'bg-orange-500 bg-opacity-40 border-orange-400',
  'bg-pink-500 bg-opacity-40 border-pink-400',
  'bg-teal-500 bg-opacity-40 border-teal-400',
  'bg-indigo-500 bg-opacity-40 border-indigo-400',
  'bg-red-500 bg-opacity-40 border-red-400',
  'bg-cyan-500 bg-opacity-40 border-cyan-400',
  'bg-lime-500 bg-opacity-40 border-lime-400',
  'bg-violet-500 bg-opacity-40 border-violet-400',
  'bg-yellow-500 bg-opacity-40 border-yellow-400'
];