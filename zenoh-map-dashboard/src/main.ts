import './assets/main.css'

import App from './App.vue'
import router from './router'
import Vue, { createApp } from '@vue/compat'
import { BootstrapVue, IconsPlugin } from 'bootstrap-vue'
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";

import { library } from "@fortawesome/fontawesome-svg-core";
import { faCar, faMotorcycle, faCircleQuestion } from "@fortawesome/free-solid-svg-icons";

library.add(faCar);
library.add(faMotorcycle);
library.add(faCircleQuestion);

import 'bootstrap/dist/css/bootstrap.css'
import 'bootstrap-vue/dist/bootstrap-vue.css'

// Make BootstrapVue available throughout your project
Vue.use(BootstrapVue)
// Optionally install the BootstrapVue icon components plugin
Vue.use(IconsPlugin)

const app = createApp(App)
app.component("font-awesome-icon", FontAwesomeIcon)

app.use(router)

app.mount('#app')
