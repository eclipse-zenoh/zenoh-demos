<template>
  <div class="container-fluid">
    <div class="row">
      <div class="col-12 d-flex pb-2">
        <b-input id="rest_api" type="text" v-model="endpoint" />
        <b-button @click="connect">Connect</b-button>
      </div>
    </div>

    <div
      class="alert"
      :class="{
        'alert-primary': status === 'primary',
        'alert-warning': status == 'warning',
        'alert-danger': status === 'danger'
      }"
      role="alert"
      :ref="alertDiv"
    >
      {{ alertMsg }}
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
import { DistanceAlert } from '../types/DistanceAlert.vue'

const endpoint = ref('http://3.71.106.121:8000/')
const session = ref(null)
const alertDiv = ref(null)
const carsMap = ref(new Map())
const cars = ref(new Array())
const alertMsg = ref('No Alerts')
const status = ref('primary')

function connect() {
  session.value = new Zenoh(endpoint)
  session.value.subscribe('demo/tracker/mobs/*', onMobsData)
  session.value.subscribe('demo/tracker/alert/distance', onAlertData)
}

function onMobsData(sample) {
  const data = sample.data
  const zsample = JSON.parse(data)
  let value = zsample.value
  if (zsample.encoding == 'application/octet-stream') {
    value = atob(value)
  } else if (zsample.encoding == 'applications/json') {
    value = JSON.parse(value)
  }
  let timestamp = new Date(zsample.time.split('/')[0]).getTime()

  value.timestamp = timestamp

  carsMap.value.set(value.id, value)
  updateCars()
}

function onAlertData(sample) {
  const data = sample.data
  const zsample = JSON.parse(data)
  const alert = zsample.value

  alertMsg.value = `WARNING ${alert.ida} and ${alert.idb} are too close! Distance: ${alert.distance}`
  if (alert.kind === 'ALERT')
    status.value ='warning'
  else
    status.value ='danger'

  setTimeout(() => {
    clearAlert()
  }, 2500)
}

function removeDead() {
  const now = Date.now()
  let newMap = new Map()

  for (const [id, car] of carsMap.value) if (car.timestamp + 10000 > now) newMap.set(id, car)

  carsMap.value = newMap
  updateCars()
}

function updateCars() {
  cars.value = new Array()
  for (const [_, car] of carsMap.value) cars.value.push(car)
}

function clearAlert() {
  alertMsg.value = 'No Alerts'
  status.value ='primary'
  // alertDiv.value.class="alert alert-success"
}

onMounted(() => {
  setInterval(() => {
    removeDead()
  }, 5000)
})

// here we should subscribe to Zenoh data and update the points in the map
</script>
