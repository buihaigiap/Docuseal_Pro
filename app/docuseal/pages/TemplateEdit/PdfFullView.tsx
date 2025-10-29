import React from 'react';
import PdfDisplay from '../../components/PdfDisplay';
import SignatureRenderer from '../../components/SignatureRenderer';
import { partnerColorClasses, fieldTools } from '../../components/PdfFieldEditor/constants';
import { getFieldClass } from '../../components/PdfFieldEditor/utils';
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
}

const PdfFullView: React.FC<PdfFullViewProps> = ({
  templateInfo,
  fields,
  page,
  onPageChange,
  onFieldClick,
  texts,
  token
}) => {
  console.log('fields' , fields)
  return (
    <div>
      {templateInfo && (
        <PdfDisplay
          filePath={templateInfo.document.url}
          token={token}
          page={page}
          onPageChange={onPageChange}
        >
          {fields.filter(f => f?.position?.page === page)?.map(field => {
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
                  display: 'flex',
                  alignItems: 'center',
                  fontSize: '16px',
                  color: 'black',
                  fontWeight: 'bold'
                }}
                onClick={() => onFieldClick(field)}
                title={field.name}
              >
                {texts[field.id] ? (
                  field.field_type === 'signature' || field.field_type === "initials" ? (
                    <SignatureRenderer
                      fieldType={field.field_type}
                      data={texts[field.id]}
                      width={field.position.width * 600}
                      height={field.position.height * 800}
                    />
                  ) :
                  field.field_type === 'image' ? (
                    <div className="w-full h-full">
                      <img 
                        src={texts[field.id]} 
                        alt="Uploaded" 
                        className="w-full h-full "
                      />
                    </div>
                  ) :
                  field.field_type === 'file' ? (
                    <div className="w-full h-full flex items-center overflow-hidden">
                      <span className="text-xs " title={decodeURIComponent(texts[field.id].split('/').pop() || 'File')}>
                        {decodeURIComponent(texts[field.id].split('/').pop() || 'File')}
                      </span>
                    </div>
                  ) :
                  field.field_type === 'checkbox' ? (
                    <div className="w-full h-full">
                      <div className={`w-full h-full   ${texts[field.id] === 'true' ? 'bg-indigo-600' : ''}`}>
                        {texts[field.id] === 'true' && (
                          <svg className="w-full h-full text-white p-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                          </svg>
                        )}
                      </div>
                    </div>
                  ) :
                  field.field_type === 'radio' ? (
                    <div className="w-full h-full flex items-center justify-center">
                      <span className="truncate text-sm">{texts[field.id]}</span>
                    </div>
                  ) :
                  field.field_type === 'multiple' ? (
                    <div className="w-full h-full flex items-center justify-center px-2 overflow-hidden">
                      <span className="text-xs truncate">{texts[field.id]}</span>
                    </div>
                  ) :
                  field.field_type === 'select' ? (
                    <div className="w-full h-full flex items-center justify-center">
                      <span className="truncate text-sm">{texts[field.id]}</span>
                    </div>
                  ) :
                  field.field_type === 'cells' ? (
                    <div className="w-full h-full grid overflow-hidden" style={{ gridTemplateColumns: field.options?.widths?.map((w: number) => `${w}fr`).join(' ') || '1fr 1fr 1fr' }}>
                      {Array.from({length: field.options?.columns || 3}, (_, i) => {
                        const char = texts[field.id]?.[i] || '';
                        return (
                          <div key={i} className=" flex items-center justify-end text-base font-bold ">
                            {char}
                          </div>
                        );
                      })}
                    </div>
                  ) :
                  texts[field.id].length > 10 ? texts[field.id].slice(0, 10) + '...' : texts[field.id]
                ) : (
                  // Khi chưa có dữ liệu, hiển thị preview
                  field.field_type === 'cells' ? (
                    <div className="w-full h-full grid overflow-hidden" style={{ gridTemplateColumns: field.options?.widths?.map((w: number) => `${w}fr`).join(' ') || '1fr 1fr 1fr' }}>
                      {Array.from({length: field.options?.columns || 3}, (_, i) => (
                        <div key={i} className="border border-gray-400 flex items-center justify-center text-xs bg-white bg-opacity-50">
                          {i + 1}
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className='text-center w-full h-full flex items-center justify-center bg-white bg-opacity-50'>
                      {fieldTools.find(ft => ft.type === field.field_type)?.iconComponent('w-6 h-6')}
                    </div>
                  )
                )}
              </div>
            );
          })}
        </PdfDisplay>
      )}
    </div>
  );
};

export default PdfFullView;