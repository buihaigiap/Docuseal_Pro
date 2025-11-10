import { useState, useRef, useEffect } from "react";
import PdfDisplay, { PdfDisplayRef } from "./PdfDisplay";
import SignatureRenderer from "./SignatureRenderer";

interface DocumentViewerProps {
  documentUrl?: string;
  filePath?: string;
  token?: string | null;
  fields?: any[];
  texts?: Record<number, string>;
  onFieldClick?: (field: any) => void;
  page?: number;
  onPageChange?: (page: number) => void;
  scale?: number;
  showDebug?: boolean;
  submitterId?: number;
  submitterEmail?: string;
}

const DocumentViewer: React.FC<DocumentViewerProps> = ({
  documentUrl,
  filePath,
  token,
  fields = [],
  texts = {},
  onFieldClick,
  page,
  onPageChange,
  scale: initialScale = 1.5,
  submitterId,
  submitterEmail,
}) => {
  console.log('fields' , fields)
  const [currentPage, setCurrentPage] = useState(page || 1);
  const [scale, setScale] = useState(initialScale);
  const pdfRef = useRef<PdfDisplayRef>(null);
  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage); 
    if (onPageChange) onPageChange(newPage);
  };

  const updateScale = () => {
    if (pdfRef.current) {
      const displayedHeight = pdfRef.current.getCanvasClientHeight();
      const pdfHeight = 792; // Letter height in points
      if (pdfHeight > 0) {
        setScale(displayedHeight / pdfHeight);
      }
    }
  };

  // Update scale when window resizes
  useEffect(() => {
    const handleResize = () => {
      updateScale();
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  return (
    <div className="flex flex-col items-center">
      <PdfDisplay
        documentUrl={documentUrl}
        filePath={filePath}
        token={token}
        scale={initialScale}
        page={currentPage}
        onPageChange={handlePageChange}
        onLoad={updateScale}
        ref={pdfRef}
      >
        {fields.filter(f => f?.position?.page === currentPage)?.map((f, index) => {
          // Position data is in absolute pixels, need to multiply by scale
          const isNarrow = f.position.h > 0 && (f.position.w / f.position.h) > 6;
          return (
            <div
              key={f.id}
              className={`absolute ${(f as any).signature_value ? '' : 'border-2 border-blue-500 bg-blue-500 bg-opacity-20 hover:bg-opacity-40 cursor-pointer'}`}
              style={{
                left: `${f.position.x * scale}px`,
                top: `${f.position.y * scale}px`,
                width: `${f.position.width * scale}px`,
                height: `${f.position.height * scale}px`,
              }}
            onClick={() => !(f as any).signature_value && onFieldClick && onFieldClick(f)}
          >
            <div className={`w-full h-full flex ${(f as any).field_type === "initials" ? " items-start" : "items-center "} text-md text-black font-semibold`}>
              {(f as any).signature_value ? (
                (f as any).field_type === 'image' ? (
                  <img 
                    src={(f as any).signature_value} 
                    alt="Uploaded image" 
                    className="object-contain mx-auto w-full h-full"
                  />
                ) : (f as any).signature_value.startsWith('data:image/') ? (
                  <div className={`flex justify-between w-full h-full gap-1 overflow-hidden ${isNarrow ? 'flex-row' : 'flex-col'}`}>
                    <div className={`flex overflow-hidden ${isNarrow ? 'w-1/2' : 'flex-grow'}`} style={{ minHeight: '50%' }}>
                      <img src={(f as any).signature_value} alt="Signature" className="object-contain mx-auto max-w-full max-h-full" />
                    </div>
                  </div>
                ) : (f as any).signature_value.startsWith('blob:') || (f as any).signature_value.startsWith('http') ? (
                  <div className={`flex justify-between w-full h-full gap-1 overflow-hidden ${isNarrow ? 'flex-row' : 'flex-col'}`}>
                    <div className={`flex overflow-hidden ${isNarrow ? 'w-1/2' : 'flex-grow'}`} style={{ minHeight: '50%' }}>
                      <img src={(f as any).signature_value} alt="Signature" className="object-contain mx-auto max-w-full max-h-full" />
                    </div>
                  </div>
                ) : (f as any).signature_value.startsWith('[') || (f as any).signature_value.startsWith('{') ? (
                  <div className={`flex justify-between w-full h-full gap-1 overflow-hidden ${isNarrow ? 'flex-row' : 'flex-col'}`}>
                    <div className={`flex overflow-hidden ${isNarrow ? 'w-1/2' : 'flex-grow'}`} style={{ minHeight: '50%' }}>
                      <SignatureRenderer 
                        data={(f as any).signature_value} 
                        width={f.position.width * scale} 
                        height={f.position.height * scale}
                        submitterId={submitterId}
                        submitterEmail={submitterEmail}
                        reason={(f as any).reason}
                      />
                    </div>
                  </div>
                ) : (f as any).field_type === 'checkbox' ? (
                  (f as any).signature_value === 'true' ? (
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" className="w-full h-full"><path d="M19 3H5C3.9 3 3 3.9 3 5V19C3 20.1 3.9 21 5 21H19C20.1 21 21 20.1 21 19V5C21 3.9 20.1 3 19 3ZM10 17L5 12L6.41 10.59L10 14.17L17.59 6.58L19 8L10 17Z" fill="currentColor"/></svg>
                  ) : (
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" className="w-full h-full"><rect x="3" y="3" width="18" height="18" rx="2" stroke="currentColor" strokeWidth="2"/></svg>
                  )
                ) : (f as any).field_type === 'file' ? (
                  <a 
                    href={(f as any).signature_value} 
                    download 
                    className="text-black underline cursor-pointer text-xs"
                    onClick={(e) => e.stopPropagation()}
                  >
                    {decodeURIComponent((f as any).signature_value.split('/').pop() || 'File')}
                  </a>
                ) : (
                  <span 
                      className="text-sm"
                      style={(f as any).field_type === "initials" ? 
                        { 
                          display: 'block',
                          position: 'absolute',
                          height: '100%', 
                          fontFamily: 'Helvetica, Arial, sans-serif', 
                          fontStyle: 'normal', 
                          fontWeight: 'normal', 
                          lineHeight: `${f.position.height}px` } : { whiteSpace: 'pre', fontFamily: 'Helvetica, Arial, sans-serif' }}>{(f as any).signature_value}
                    </span>
                )
              ) : texts[f.id] ? (
                (f as any).field_type === 'image' ? (
                  <img 
                    src={texts[f.id]} 
                    alt="Uploaded image" 
                    className="object-contain mx-auto w-full h-full"
                  />
                ) : texts[f.id].startsWith('data:image/') ? (
                  <div className={`flex justify-between w-full h-full gap-1 overflow-hidden ${isNarrow ? 'flex-row' : 'flex-col'}`}>
                    <div className={`flex overflow-hidden ${isNarrow ? 'w-1/2' : 'flex-grow'}`} style={{ minHeight: '50%' }}>
                      <img src={texts[f.id]} alt="Signature" className="object-contain mx-auto max-w-full max-h-full" />
                    </div>
                  </div>
                ) : texts[f.id].startsWith('blob:') || texts[f.id].startsWith('http') ? (
                  <div className={`flex justify-between w-full h-full gap-1 overflow-hidden ${isNarrow ? 'flex-row' : 'flex-col'}`}>
                    <div className={`flex overflow-hidden ${isNarrow ? 'w-1/2' : 'flex-grow'}`} style={{ minHeight: '50%' }}>
                      <img src={texts[f.id]} alt="Signature" className="object-contain mx-auto max-w-full max-h-full" />
                    </div>
                  </div>
                ) : texts[f.id].startsWith('[') || texts[f.id].startsWith('{') ? (
                  <div className={`flex justify-between w-full h-full gap-1 overflow-hidden ${isNarrow ? 'flex-row' : 'flex-col'}`}>
                    <div className={`flex overflow-hidden ${isNarrow ? 'w-1/2' : 'flex-grow'}`} style={{ minHeight: '50%' }}>
                      <SignatureRenderer 
                        data={texts[f.id]} 
                        width={f.position.width * scale} 
                        height={f.position.height * scale}
                      />
                    </div>
                  </div>
                ) : (f as any).field_type === 'multiple' ? (
                  <div className="w-full h-full flex items-center text-sm font-semibold">
                    {texts[f.id] ? texts[f.id].split(',').join(', ') : `Select ${(f as any).name}`}
                  </div>
                ) : (f as any).field_type === 'cells' ? (
                  <div className="w-full h-full grid" style={{ gridTemplateColumns: `repeat(${(f as any).options?.columns || 1}, 1fr)` }}>
                    {(texts[f.id] || '').split('').map((char: string, i: number) => (
                     <div
                        key={i}
                        className="flex items-center justify-end text-lg text-black font-normal"
                      >
                        {char}
                      </div>
                    ))}
                    {/* Fill empty cells */}
                    {Array.from({length: ((f as any).options?.columns || 1) - (texts[f.id] || '').length}, (_, i) => (
                      <div key={`empty-${i}`} className="border border-gray-400 flex items-center justify-start text-xs text-black font-normal">
                      </div>
                    ))}
                  </div>
                ) : (f as any).field_type === 'file' ? (
                  <a 
                    href={texts[f.id]} 
                    download 
                    className="text-black underline cursor-pointer text-xs"
                    onClick={(e) => e.stopPropagation()}
                  >
                    {decodeURIComponent(texts[f.id].split('/').pop() || 'File')}
                  </a>
                ) : (
                  <span className="text-sm" style={(f as any).field_type === "initials" ? 
                    { 
                      display: 'block',
                      position: 'absolute',
                      height: '100%', 
                      fontFamily: 'Helvetica, Arial, sans-serif', 
                      fontStyle: 'normal', 
                      fontWeight: 'normal', 
                      lineHeight: `${f.position.height}px` } : { fontFamily: 'Helvetica, Arial, sans-serif' }}>{texts[f.id]}</span>
                )
              ) : (f as any).field_type === 'radio' ? (
                <div className="w-full h-full flex items-center text-sm font-semibold">
                  {texts[f.id] || `Select ${(f as any).name}`}
                </div>
              ) : (
                <span className="text-sm">{f.name}</span>
              )}
            </div>
          </div>
        );
        })}
      </PdfDisplay>
      {/* <Box>

      </Box> */}
    </div>
  );
};

export default DocumentViewer;
