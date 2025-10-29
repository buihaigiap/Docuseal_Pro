

export const freePlanData: any = {
  id: 'free',
  name: 'Free',
  price: 0,
  period: 'forever',
  features: [
    { text: '5 signature requests / month' },
    { text: '1 user' },
    { text: 'Basic fields' },
    { text: 'Audit trail' },
  ],
  buttonText: 'Get Started',
};

export const proPlanData: any = {
  id: 'pro',
  name: 'Pro',
  pricing: {
    monthly: 20,
    yearly: 200, 
  },
  features: [
    { text: 'Unlimited signature requests' },
    { text: 'Conditional fields and formulas' },
    { text: 'Your company logo' },
    { text: 'User roles and teams', info: 'Manage team member permissions.' },
    { text: 'Custom email content' },
    { text: 'Bulk send from CSV, XLSX spreadsheet', info: 'Send to multiple recipients at once.' },
    { text: 'Automated reminders' },
    { text: 'SSO / SAML' },
    { text: 'Webhooks' },
    { text: 'Invitation and verification via SMS', info: 'Additional charges may apply.' },
    { text: 'Connect your Gmail or Outlook email' },
    { text: 'API and Embedding', info: 'Integrate into your own applications.' },
  ],
  buttonText: 'Upgrade to Pro',
};

export const plans: any[] = [freePlanData, proPlanData];
