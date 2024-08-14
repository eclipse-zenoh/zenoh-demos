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
import { DistanceAlert } from '../types/DistanceAlert.vue'
import { Config, Subscriber, Session, Sample, Encoding } from '@ZettaScaleLabs/zenoh-ts'
import { l } from 'vite/dist/node/types.d-aGj9QkWt'

// Number of NTP fraction per second (2^32)
const FRAC_PER_SEC: bigint = 1n << 32n
// Bit-mask for the fraction of a second part within an NTP timestamp
const FRAC_MASK: bigint = 0xffff_ffffn

const endpoint = ref('ws/172.17.0.1:9000')
const session = ref(null)
const mobsSubscriber = ref(null)
const alertSubscriber = ref(null)
const alertDiv = ref(null)
const carsMap = ref(new Map())
const cars = ref(new Array())
const alertMsg = ref('No Alerts')
const status = ref('primary')

async function connect() {
  session.value = await Session.open(Config.new(endpoint.value))
  mobsSubscriber.value = await session.value.declare_subscriber('demo/tracker/mobs/*', onMobsData)
  alertSubscriber.value = await session.value.declare_subscriber(
    'demo/tracker/alert/distance',
    onAlertData
  )
}

async function onMobsData(sample) {
  const data = sample.payload().payload()
  //const ke = sample.keyexpr()

  // const zsample = JSON.parse(data)
  // let value = zsample.value
  // if (sample.encoding() == Encoding.APPLICATION_JSON()) {
  let utf8Encode = new TextDecoder()
  let data_string = utf8Encode.decode(data)
  let value = JSON.parse(data_string)
  let timestamp = parseTimesamp(sample.timestamp()).getTime()
  //console.log(`Received: [${timestamp}] ${ke} - ${JSON.stringify(value)}`)
  // }
  // let timestamp = new Date(zsample.time.split('/')[0]).getTime()

  value.timestamp = timestamp

  carsMap.value.set(value.id, value)
  updateCars()
}

function onAlertData(sample) {
  const data = sample.payload().payload()
  let utf8Encode = new TextDecoder()
  let data_string = utf8Encode.decode(data)
  let alert = JSON.parse(data_string)

  alertMsg.value = `WARNING ${alert.ida} and ${alert.idb} are too close! Distance: ${alert.distance}`
  if (alert.kind === 'ALERT') status.value = 'warning'
  else status.value = 'danger'

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
  status.value = 'primary'
  // alertDiv.value.class="alert alert-success"
}

function parseTimesamp(zenoh_timestamp: string) {
  try {
    const chunks = zenoh_timestamp.split('/')
    const ntp64: bigint = BigInt(chunks[0])
    const unixTime: number = asSecsF64(ntp64) * 1000
    const date = new Date(unixTime)
    return date
  } catch (e) {
    // If date cannot be parsed then return current time
    return new Date()
  }
}

function asSecs(ntp: bigint): number {
  return Number(ntp >> 32n)
}
function asSecsF64(ntp: bigint): number {
  const secs: number = asSecs(ntp)
  const subsec: number = Number(ntp & FRAC_MASK) / Number(FRAC_PER_SEC)
  return secs + subsec
}

onMounted(() => {
  setInterval(() => {
    removeDead()
  }, 5000)
})

// here we should subscribe to Zenoh data and update the points in the map
</script>
