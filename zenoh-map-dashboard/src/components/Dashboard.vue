<template>
  <div class="container-fluid">
    <div class="row">
      <div class="col-12 d-flex pb-2">
        <b-input id="rest_api" type="text" v-model="endpoint" />
        <b-button @click="connect">Connect</b-button>
      </div>
    </div>
    <div class="row">
      <div class="col-12">
        <Map :markers="cars"></Map>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import Map from './Map.vue'
import { Zenoh } from 'zenoh-js'

const endpoint = ref('http://3.71.106.121:8000/')
const session = ref(null)
const cars = ref([])

function connect() {
  session.value = new Zenoh(endpoint)
  session.value.subscribe('demo/cardata', onData)
}

function onData(sample) {
  const data = sample.data
  const zsample = JSON.parse(data)
  let value = zsample.value
  if (zsample.encoding == 'application/octet-stream') {
    value = atob(value)
  } else if (zsample.encoding == 'applications/json') {
    value = JSON.parse(value)
  }
  //console.log('[' + zsample.key + ', ' + zsample.encoding + ', ' + zsample.time + '] - ' + value)
  // console.log(value)
  // dataReceived.value = value
  // cars.value = []
  cars.value = value
}

// here we should subscribe to Zenoh data and update the points in the map
</script>
