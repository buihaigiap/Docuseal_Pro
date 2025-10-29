import React, { useState, useEffect } from 'react';
import { Typography, Box, Button, Table, TableBody, TableCell, TableContainer, TableHead, TableRow, Paper, Dialog, DialogTitle, DialogContent, DialogActions, TextField, Select, MenuItem, FormControl, InputLabel, Chip } from '@mui/material';
import CreateTemplateButton from '@/components/CreateTemplateButton';
import upstashService from '../../../ConfigApi/upstashService';
import toast from 'react-hot-toast';

const UsersSettings = () => {
  const [open, setOpen] = useState(false);
  const [formData, setFormData] = useState({ name: '', email: '', role: '' });
  const [loading, setLoading] = useState(false);
  const [users, setUsers] = useState([]);
  const [fetchLoading, setFetchLoading] = useState(true);
  const roles = ['admin', 'editor', 'member', 'agent', 'viewer'];

  useEffect(() => {
    const fetchUsers = async () => {
      try {
        const response = await upstashService.getUserAccounts();
        setUsers(response.data);
      } catch (err) {
        toast.error('Failed to fetch users');
      } finally {
        setFetchLoading(false);
      }
    };
    fetchUsers();
  }, []);

  const tableColumns = ['Name', 'Email', 'Role', 'Status'];
  const tableKeys = ['name', 'email', 'role', 'status'];

  const handleClickOpen = () => {
    setOpen(true);
  };

  const handleClose = () => {
    setOpen(false);
    setFormData({ name: '', email: '', role: '' });
  };

  const handleChange = (e: any) => {
    setFormData({ ...formData, [e.target.name]: e.target.value });
  };

  const handleSubmit = async () => {
    setLoading(true);
    try {
      const response = await upstashService.addTeam({
        email: formData.email,
        name: formData.name,
        role: formData.role
      });
      if (response.success) {
        toast.success('User added successfully');
        // Refetch users
        const fetchResponse = await upstashService.getUserAccounts();
        if (fetchResponse.success) {
          setUsers(fetchResponse.data);
        }
        handleClose();
      } else {
        toast.error(response.message || 'Failed to add user');
      }
    } catch (error) {
      console.error('Error adding user:', error);
      toast.error('Failed to add user');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box>
      <Box display='flex' alignItems='center' justifyContent='space-between' mb={3}>
        <Typography variant="h5">
          Users Settings
        </Typography>
        
        <CreateTemplateButton
            onClick={handleClickOpen}
            text="New User"
            background="linear-gradient(135deg, #4F46E5 0%, #7C3AED 100%)"
        />
      </Box>

      <TableContainer component={Paper} sx={{ color: 'white', bgcolor: 'transparent' }}>
        <Table sx={{ '& .MuiTableCell-root': { borderBottom: 'none'  } }}>
          <TableHead>
            <TableRow>
              {tableColumns.map((column) => (
                <TableCell key={column} sx={{ color: 'white', fontWeight: 'bold' }}>
                  {column}
                </TableCell>
              ))}
            </TableRow>
          </TableHead>
          <TableBody>
            {fetchLoading ? (
              <TableRow>
                <TableCell colSpan={tableColumns.length} sx={{ color: 'white', textAlign: 'center' }}>
                  Loading...
                </TableCell>
              </TableRow>
            ) : (
              users.map((user, index) => (
                <TableRow key={index}>
                  {tableKeys.map((key) => (
                    <TableCell key={key} sx={{ color: 'white' }}>
                      {key === 'status' ? (
                        <Chip
                          label={(user[key as keyof typeof user] as string).charAt(0).toUpperCase() + (user[key as keyof typeof user] as string).slice(1)}
                          color={user[key as keyof typeof user] === 'pending' ? 'warning' : 'success'}
                          size="small"
                        />
                      ) : key === 'role' ? (
                        (user[key as keyof typeof user] as string).charAt(0).toUpperCase() + (user[key as keyof typeof user] as string).slice(1)
                      ) : (
                        user[key as keyof typeof user]
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </TableContainer>

      <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
        <DialogTitle>Add New User</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            name="name"
            label="Name"
            type="text"
            fullWidth
            variant="outlined"
            value={formData.name}
            onChange={handleChange}
            sx={{ mb: 2, '& .MuiInputLabel-root': { color: 'white' }, '& .MuiOutlinedInput-root': { color: 'white', '& fieldset': { borderColor: 'white' } } }}
          />
          <TextField
            margin="dense"
            name="email"
            label="Email"
            type="email"
            fullWidth
            variant="outlined"
            value={formData.email}
            onChange={handleChange}
            sx={{ mb: 2, '& .MuiInputLabel-root': { color: 'white' }, '& .MuiOutlinedInput-root': { color: 'white', '& fieldset': { borderColor: 'white' } } }}
          />
          <FormControl fullWidth margin="dense" sx={{ mb: 2 }}>
            <InputLabel sx={{ color: 'white' }}>Role</InputLabel>
            <Select
              name="role"
              value={formData.role}
              onChange={(e) => setFormData({ ...formData, role: e.target.value })}
              sx={{ color: 'white', '& .MuiOutlinedInput-notchedOutline': { borderColor: 'white' } }}
              MenuProps={{
                PaperProps: {
                  sx: {
                    bgcolor: '#1a1a1a',
                    '& .MuiMenuItem-root': {
                      color: 'white',
                      '&:hover': {
                        bgcolor: 'grey.700',
                      },
                    },
                  },
                },
              }}
            >
              {roles.map((role) => (
                <MenuItem key={role} value={role}>
                  {role.charAt(0).toUpperCase() + role.slice(1)}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </DialogContent>
        <DialogActions>
            <Button
                onClick={handleClose}
                variant="outlined"
                color="inherit"
                disabled={loading}
                sx={{
                  borderColor: "#475569", // slate-600
                  color: "#cbd5e1",
                  textTransform: "none",
                  fontWeight: 500,
                  "&:hover": { backgroundColor: "#334155" },
                }}
            >
              Cancel
            </Button>
            <CreateTemplateButton
                 onClick={handleSubmit}
                text="Submit"
                loading={loading}
            />
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default UsersSettings;