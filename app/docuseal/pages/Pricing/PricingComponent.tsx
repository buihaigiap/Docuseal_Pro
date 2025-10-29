import React, { useState } from 'react';
// import InfoIcon from './icons/InfoIcon';
import { CircleCheck , BadgeQuestionMark } from 'lucide-react';
const BillingToggle: React.FC<{ billingCycle: 'monthly' | 'yearly', setBillingCycle: (cycle: 'monthly' | 'yearly') => void }> = ({ billingCycle, setBillingCycle }) => {
  return (
    <div className="relative">
      <div className="absolute -top-7 right-0 text-xs bg-purple-500 text-white px-2 py-0.5 rounded-full font-medium shadow-md shadow-purple-500/30">2 months free</div>
      <div className="relative bg-black/20 rounded-full p-1 flex items-center w-52">
        <span
          className="absolute top-1 left-1 bottom-1 w-[calc(50%-4px)] bg-gradient-to-r from-purple-600 to-blue-500 rounded-full transition-transform duration-300 ease-in-out"
          style={{ transform: `translateX(${billingCycle === 'yearly' ? '100%' : '0'})` }}
        ></span>
        <button
          onClick={() => setBillingCycle('monthly')}
          className="relative z-10 w-1/2 py-1.5 text-sm font-semibold rounded-full transition-colors duration-300"
        >
          Monthly
        </button>
        <button
          onClick={() => setBillingCycle('yearly')}
          className="relative z-10 w-1/2 py-1.5 text-sm font-semibold rounded-full transition-colors duration-300"
        >
          Yearly
        </button>
      </div>
    </div>
  );
};

const PricingComponent: React.FC<{ plan: any, onSubscribe: (plan: any, price?: number, period?: 'monthly' | 'yearly') => void }> = ({ plan, onSubscribe }) => {

  if (plan.id === 'pro') {
    const [billingCycle, setBillingCycle] = useState<'monthly' | 'yearly'>('monthly');
    const currentPricePerMonth = billingCycle === 'monthly' ? plan.pricing.monthly : plan.pricing.yearly / 12;
    const periodText = '/ user / month';
    const totalPrice = billingCycle === 'monthly' ? plan.pricing.monthly : plan.pricing.yearly;

    const handleSubscribeClick = () => {
      onSubscribe(plan, totalPrice, billingCycle);
    };

    return (
      <div className="relative bg-white/5 backdrop-blur-lg border border-white/10 rounded-3xl p-6 sm:p-8 flex flex-col h-full text-white shadow-2xl shadow-black/20">
        <div className="absolute -inset-0.5 bg-gradient-to-r from-purple-600 to-blue-500 rounded-3xl blur-lg opacity-20 group-hover:opacity-40 transition duration-1000 group-hover:duration-200 animate-tilt"></div>
        <div className='relative'>
            <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
            <h2 className="text-5xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-purple-400 to-pink-400">{plan.name}</h2>
            <BillingToggle billingCycle={billingCycle} setBillingCycle={setBillingCycle} />
            </div>

            <div className="my-4">
            <span className="text-5xl font-bold">${Math.floor(currentPricePerMonth)}</span>
            <span className="text-lg text-slate-400 ml-2">{periodText}</span>
            {billingCycle === 'yearly' && (
                <p className="text-sm text-slate-400 mt-1">Billed as ${plan.pricing.yearly} per year.</p>
            )}
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-4 mb-10">
            {plan.features.map((feature, index) => (
                <div key={index} className="flex items-start">
                  <div className="flex-shrink-0 w-6 h-6 rounded-full bg-purple-500/20 flex items-center justify-center mr-3">
                    <CircleCheck className="w-4 h-4 text-white" />
                  </div>
                  <span className="flex items-center gap-1.5 text-slate-300">
                    {feature.text}
                    {feature.info && (
                    <span className="group relative">
                        <BadgeQuestionMark className="w-4 h-4 text-slate-500 cursor-pointer" />
                        <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 w-max max-w-xs bg-slate-800 text-white text-xs rounded py-1 px-2 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10 border border-white/10">
                        {feature.info}
                        </span>
                    </span>
                    )}
                </span>
                </div>
            ))}
            </div>

            <div className="mt-auto">
            <button 
                onClick={handleSubscribeClick}
                className="w-full bg-gradient-to-r from-purple-600 to-blue-500 text-white text-lg font-semibold py-4 rounded-xl hover:shadow-lg hover:shadow-purple-500/40 transform hover:-translate-y-1 transition-all duration-300"
            >
                {plan.buttonText}
            </button>
            </div>
        </div>
      </div>
    );
  }

  if (plan.id === 'free') {
    const handleSubscribeClick = () => {
      onSubscribe(plan);
    };
    
    return (
      <div className="bg-white/5 backdrop-blur-lg border border-white/10 rounded-3xl p-6 sm:p-8 flex flex-col h-full text-white shadow-2xl shadow-black/20">
        <h2 className="text-5xl font-bold text-slate-400">{plan.name}</h2>
        
        <div className="my-4">
          <span className="text-5xl font-bold text-white">$0</span>
          <span className="text-lg text-slate-400 ml-2">/ {plan.period}</span>
        </div>

        <div className="space-y-4 mb-10">
          {plan.features.map((feature, index) => (
            <div key={index} className="flex items-start">
               <div className="flex-shrink-0 w-6 h-6 rounded-full bg-slate-500/20 flex items-center justify-center mr-3">
                       <CircleCheck className="w-4 h-4 text-white" />
              </div>
              <span className="text-slate-300">{feature.text}</span>
            </div>
          ))}
        </div>
        
        <div className="mt-auto">
          <button
            onClick={handleSubscribeClick}
            className="w-full bg-white/10 text-white border-2 border-white/20 text-lg font-semibold py-4 rounded-xl hover:bg-white/20 transition-colors duration-300"
          >
            {plan.buttonText}
          </button>
        </div>
      </div>
    );
  }

  return null;
};

export default PricingComponent;