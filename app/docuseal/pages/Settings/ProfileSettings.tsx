import { useState, useEffect } from 'react';
import { Typography, Box, TextField, Button } from '@mui/material';
import upstashService from '../../ConfigApi/upstashService';
import toast from 'react-hot-toast';

const ProfileSettings = () => {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');

  useEffect(() => {
    const fetchUser = async () => {
      try {
        const res = await upstashService.getMe();
        setName(res.data.name);
        setEmail(res.data.email);
      } catch (err) {
        toast.error('Failed to load user data');
      }
    };
    fetchUser();
  }, []);

  const handleUpdateProfile = async (e) => {
    e.preventDefault();
    try {
      await upstashService.updateProfile({ name, email });
      toast.success('Profile updated successfully');
    } catch (err) {
      toast.error('Failed to update profile');
    }
  };

  const handleChangePassword = async (e) => {
    e.preventDefault();
    if (newPassword !== confirmPassword) {
      toast.error('New passwords do not match');
      return;
    }
    try {
      await upstashService.changePassword({ current_password: currentPassword, new_password: newPassword });
      toast.success('Password changed successfully');
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
    } catch (err) {
      toast.error('Failed to change password');
    }
  };

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" sx={{ mb: 3 }}>
        Profile Settings
      </Typography>
      <Box sx={{ display: 'flex', flexDirection: { xs: 'column', md: 'row' }, gap: 3 }}>
        <div className="bg-white/5 border border-white/10 rounded-lg p-4 flex-1">
          <Typography variant="h6" sx={{ mb: 2 }}>
            Update Profile
          </Typography>
          <Box component="form" onSubmit={handleUpdateProfile}>
            <TextField
              fullWidth
              label="Name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              sx={{ mb: 2 }}
            />
            <TextField
              fullWidth
              label="Email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              sx={{ mb: 2 }}
            />
            <Button type="submit" variant="contained" color="primary">
              Update Profile
            </Button>
          </Box>
        </div>
        <div className="bg-white/5 border border-white/10 rounded-lg p-4 flex-1">
          <Typography variant="h6" sx={{ mb: 2 }}>
            Change Password
          </Typography>
          <Box component="form" onSubmit={handleChangePassword}>
            <TextField
              fullWidth
              type="password"
              label="Current Password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              sx={{ mb: 2 }}
            />
            <TextField
              fullWidth
              type="password"
              label="New Password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              sx={{ mb: 2 }}
            />
            <TextField
              fullWidth
              type="password"
              label="Confirm New Password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              sx={{ mb: 2 }}
            />
            <Button type="submit" variant="contained" color="primary">
              Change Password
            </Button>
          </Box>
        </div>
      </Box>
    </Box>
  );
};

export default ProfileSettings;