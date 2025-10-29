import React from 'react';
import { Submitter } from '../../../types';
import upstashService from '../../../ConfigApi/upstashService';
import { Trash2 } from 'lucide-react';
import toast from 'react-hot-toast';
import SubmitterItem from './SubmitterItem';

interface SigningStatusProps {
  templateInfo: any;
  handleViewSubmission: (token: string) => void;
  downloadSignedPDF: (submitter: Submitter, pdfUrl?: string) => void;
  handleDeleteSubmitter: (id: number) => void;
  fetchTemplateInfo: () => void;
  setShowInviteModal: (show: boolean) => void;
}

const SigningStatus: React.FC<SigningStatusProps> = ({
  templateInfo,
  handleViewSubmission,
  downloadSignedPDF,
  handleDeleteSubmitter,
  fetchTemplateInfo,
  setShowInviteModal,
}) => {
  return (
    <div className="mt-6">
      {templateInfo.signatures && templateInfo.signatures.length > 0 ? (
        <div className="space-y-6">
          <div className="flex justify-between items-center">
            <h2 className="text-2xl font-semibold">Signing Status</h2>
            <button onClick={() => setShowInviteModal(true)} className="px-4 py-2 font-semibold text-white bg-indigo-600 rounded-md hover:bg-indigo-700">
              Add Recipients
            </button>
          </div>
          <div className="space-y-4">
            {templateInfo.signatures.map((signature: any, signatureIndex: number) => (
              <div key={signatureIndex} className="bg-white/5 border border-white/10 rounded-lg p-4 border">
                <div className="flex items-center justify-between mb-3 text-gray-500">
                  <h3 className="text-lg font-medium text-white">
                    {signature.type === 'bulk' ? 'Bulk Signature' : 'Single Signature'}
                    <span className="text-sm ml-2">
                      ({signature.parties.length} parties)
                    </span>
                  </h3>
                  <span className={`px-3 py-1 text-xs font-bold rounded-full uppercase ${
                    signature.overall_status === 'completed'
                      ? 'bg-green-100 text-green-800'
                      : 'bg-yellow-100 text-yellow-800'
                  }`}>
                    {signature.overall_status}
                  </span>
                </div>
                {signature.type === 'bulk' ? (
                  <div className="flex justify-between items-center rounded-lg shadow-sm">
                    <div className="space-y-2 flex-1">
                      {signature.parties.map((party: any) => (
                        <SubmitterItem
                          key={party.id}
                          party={party}
                          signatureType={signature.type}
                          overallStatus={signature.overall_status}
                          showActions={false}
                        />
                      ))}
                    </div>
                    <div className="flex items-center gap-2">
                      {signature.overall_status === 'completed' && (
                      <button
                          onClick={(e) => {
                              e.stopPropagation();
                              downloadSignedPDF(signature.parties[0], templateInfo.template.file_url);
                          }}
                          className="px-3 py-1.5 text-sm font-semibold border rounded-full border-gray-500 hover:bg-gray-800 hover:text-white transition-colors flex items-center gap-1"
                      >
                          <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                          </svg>
                          DOWNLOAD
                      </button>
                      )}
                      <button
                        onClick={() => handleViewSubmission(signature.parties[0].token)}
                        className="px-3 py-1.5 text-sm font-semibold
                             border-gray-500 , border
                           rounded-full hover:bg-gray-800
                            hover:text-white transition-colors"
                      >
                        VIEW
                      </button>
                      <button
                        onClick={async (e) => {
                          e.stopPropagation();
                          if (confirm(`Are you sure you want to delete this bulk signature with ${signature.parties.length} parties?`)) {
                            try {
                              // Delete all parties in the bulk signature
                              const deletePromises = signature.parties.map(party =>
                                upstashService.deleteSubmitter(party.id)
                              );
                              await Promise.all(deletePromises);
                              toast.success('Bulk signature deleted successfully!');
                              fetchTemplateInfo();
                            } catch (err) {
                              console.error('Bulk delete error:', err);
                              toast.error('An error occurred while deleting the bulk signature.');
                            }
                          }
                        }}
                        className="p-1.5 text-gray-600 hover:text-red-600 transition-colors"
                      >
                         <Trash2 color='red'/>
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className="space-y-2">
                    {signature.parties.map((party: any) => (
                      <SubmitterItem
                        key={party.id}
                        party={party}
                        signatureType={signature.type}
                        overallStatus={signature.overall_status}
                        onDownload={downloadSignedPDF}
                        onView={handleViewSubmission}
                        onDelete={handleDeleteSubmitter}
                        pdfUrl={templateInfo.template.file_url}
                      />
                    ))}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      ) : (
        <div className="text-center py-12">
          <h2 className="text-2xl font-semibold mb-4">No Signers Yet</h2>
          <p className="text-gray-400 mb-6">Send this document to recipients for signing.</p>
          <button onClick={() => setShowInviteModal(true)} className="px-6 py-3 font-semibold text-white bg-indigo-600 rounded-md hover:bg-indigo-700">
            Send to Recipients
          </button>
        </div>
      )}
    </div>
  );
};

export default SigningStatus;