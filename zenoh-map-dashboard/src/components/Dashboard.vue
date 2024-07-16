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
        <GMap :markers="cars"></GMap>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import GMap from './GMap.vue'
import { Zenoh } from '@ZettaScaleLabs/zenoh-js'

const endpoint = ref('http://3.71.106.121:8000/')
const session = ref(null)
const carsMap = ref(new Map())
const cars = ref(new Array())

function connect() {
  session.value = new Zenoh(endpoint)
  session.value.subscribe('demo/tracker/*', onData)
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
  let timestamp = new Date(zsample.time.split('/')[0]).getTime();
  
  value.timestamp = timestamp

  carsMap.value.set(value.id, value)
  updateCars()

}

function removeDead(){
  const now = Date.now()
  let newMap = new Map()
  
  for (const [id, car] of carsMap.value) 
    if (car.timestamp + 10000 > now) newMap.set(id, car)

  carsMap.value = newMap
  updateCars()
}


function updateCars() {
  cars.value = new Array()
  for (const [_, car] of carsMap.value) 
    cars.value.push(car)
}

onMounted(() => {
  setInterval(() => {
    removeDead()
  }, 5000)
})

// here we should subscribe to Zenoh data and update the points in the map
</script>
