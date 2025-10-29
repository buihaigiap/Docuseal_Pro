import React from 'react';
import { Submitter } from '../../../types';
import { Trash2 } from 'lucide-react';

interface SubmitterItemProps {
  party: any;
  signatureType: string;
  overallStatus: string;
  onDownload?: (submitter: Submitter, pdfUrl?: string) => void;
  onView?: (token: string) => void;
  onDelete?: (id: number) => void;
  showActions?: boolean;
  pdfUrl?: string;
}

const SubmitterItem: React.FC<SubmitterItemProps> = ({
  party,
  signatureType,
  overallStatus,
  onDownload,
  onView,
  onDelete,
  showActions = true,
  pdfUrl,
}) => {
  return (
    <div className="flex items-center justify-between py-2 px-3 rounded">
      <div className="flex items-center gap-3">
        <span className={`px-3 py-1 text-xs font-bold rounded-full uppercase flex-shrink-0 ${
          party.status === 'signed' || party.status === 'completed'
            ? 'bg-cyan-400 text-gray-800'
            : 'bg-cyan-400 text-gray-800'
        }`}>
          {party.status === 'signed' || party.status === 'completed' ? 'SIGNED' : 'SENT'}
        </span>
        <div>
          <h3 className="font-medium text-white text-sm">{party.name}</h3>
          <div className="text-xs">{party.email}</div>
        </div>
      </div>
      {showActions && (
        <div className="flex items-center gap-2">
          {party.status === 'pending' && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                window.open(`${window.location.origin}/templates/${party.token}/edit`, '_blank');
              }}
              className="
              px-3 py-1.5 text-sm font-semibold
              border border-gray-500
               rounded-full hover:bg-gray-800
               hover:text-white transition-colors
                flex items-center gap-1"
            >
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
              </svg>
              SIGN NOW
            </button>
          )}
          {(party.status === 'signed' || party.status === 'completed') && overallStatus === 'completed' && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onDownload && onDownload(party, pdfUrl);
              }}
               className="
              px-3 py-1.5 text-sm font-semibold
              border border-gray-500
               rounded-full hover:bg-gray-800
               hover:text-white transition-colors
                flex items-center gap-1"
            >
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              DOWNLOAD
            </button>
          )}
          <button
            onClick={() => onView && onView(party.token)}
            className="px-3 py-1.5 text-sm font-semibold
              border border-gray-500
             rounded-full hover:bg-gray-800
              hover:text-white transition-colors"
          >
            VIEW
          </button>
          {signatureType === 'single' && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                if (confirm(`Are you sure you want to delete the submission for ${party.email}?`)) {
                  onDelete && onDelete(party.id);
                }
              }}
              className="p-1.5 text-gray-600 hover:text-red-600 transition-colors"
            >
                  <Trash2 color='red'/>
            </button>
          )}
        </div>
      )}
    </div>
  );
};

export default SubmitterItem;