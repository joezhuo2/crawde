import React from 'react';
import ReactDOM from 'react-dom/client';
import './style.css';
import { App } from './App';
import { Settings } from './Settings';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>{new URLSearchParams(location.search).has('settings') ? <Settings /> : <App />}</React.StrictMode>,
);
