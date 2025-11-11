import React from 'react';
import PdfDisplay from '../../components/PdfDisplay';
import FieldRenderer from '../../components/FieldRenderer';
import { partnerColorClasses } from '../../components/PdfFieldEditor/constants';
import { getFieldClass } from '../../components/PdfFieldEditor/utils';
import { useAuth } from '../../contexts/AuthContext';
interface TemplateInfo {
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
}

interface TemplateField {
  id: number;
  template_id: number;
  name: string;
  field_type: string;
  required: boolean;
  display_order: number;
  position: {
    x: number;
    y: number;
    width: number;
    height: number;
    page: number;
    suggested?: string;
    allow_custom?: boolean;
  };
  options?: any;
  partner?: string;
  created_at: string;
  updated_at: string;
}

interface PdfFullViewProps {
  templateInfo: TemplateInfo | null;
  fields: TemplateField[];
  page: number;
  onPageChange: (page: number) => void;
  onFieldClick: (field: TemplateField) => void;
  texts: Record<number, string>;
  token: string;
  submitterId?: number;
  submitterEmail?: string;
  reasons?: Record<number, string>;
}

const PdfFullView: React.FC<PdfFullViewProps> = ({
  templateInfo,
  fields,
  page,
  onPageChange,
  onFieldClick,
  texts,
  token,
  submitterId,
  submitterEmail,
  reasons
}) => {
  const { user } = useAuth();
  return (
    <div>
      {templateInfo && (
        <PdfDisplay
          filePath={templateInfo.document.url}
          token={token}
          page={page}
          onPageChange={onPageChange}
          // scale={1.5}
        >
          {fields.filter(f => f?.position?.page === page)?.map(field => {
            console.log('Rendering field:', field);
            return (
              <div
                key={field.id}
                className={getFieldClass(field.partner, true, partnerColorClasses)}
                style={{
                  position: 'absolute',
                  left: `${field.position.x * 100}%`,
                  top: `${field.position.y * 100}%`,
                  width: `${field.position.width * 100}%`,
                  height: `${field.position.height * 100}%`,
                  cursor: 'pointer',
                  fontSize: '16px',
                  color: 'black',
                  fontWeight: 'bold'
                }}
                onClick={() => onFieldClick(field)}
                title={field.name}
              >
                <FieldRenderer
                  field={field}
                  value={texts[field.id]}
                  defaultSignature={user?.signature}
                  defaultInitials={user?.initials}
                  submitterId={submitterId}
                  submitterEmail={submitterEmail}
                  reason={reasons?.[field.id]}
                />
              </div>
            );
          })}
        </PdfDisplay>
      )}
    </div>
  );
};

export default PdfFullView;