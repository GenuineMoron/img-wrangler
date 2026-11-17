// SPDX-License-Identifier: AGPL-3.0-or-later
import '@design/fonts.css';
import '@design/tokens.css';
import '@design/components.css';
import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

mount(App, { target: document.getElementById('app')! });
