#include <SPI.h>
#include <WiFi101.h>

extern "C"
{
    #include "zenoh-pico.h"
}

#include "turtlebot3_motor_driver.h"

#define SSID "SSID"
#define PASS "PASS"
#define MODE "client"
#define PEER ""

typedef struct TB3ModelInfo{
    const char* model_str;
    uint32_t model_info;
    uint16_t model_motor_rpm;
    float wheel_radius;
    float wheel_separation;
    float turning_radius;
    float robot_radius;
} TB3ModelInfo;

static const TB3ModelInfo burger_info = {
    "Burger",
    1,
    61,
    0.033,
    0.160,
    0.080,
    0.105,
};

static float max_linear_velocity, min_linear_velocity;
static float max_angular_velocity, min_angular_velocity;
static float goal_velocity[VelocityType::TYPE_NUM_MAX] = {0.0, 0.0};

zn_session_t *zn = NULL;
bool led_status = true;
static Turtlebot3MotorDriver motor_driver;

extern "C" void attachInterruptMultiArch(uint32_t pin, void *chip_isr, uint32_t mode)
{
    void (*_c)(void) = (void(*)(void))(chip_isr);
    attachInterrupt(pin, _c, mode);
}

extern "C" void detachInterruptMultiArch(uint32_t pin)
{
    detachInterrupt(pin);
}

/*******************************************************************************
* Callback function for cmd_vel msg
*******************************************************************************/
typedef struct
{
    double x;
    double y;
    double z;
} vector3_t;

typedef struct
{
    vector3_t linear;
    vector3_t angular;
} twist_t;

char *deserialize_float_as_f64_little_endian(double *val, char *buf)
{
    char *c_val = (char*)malloc(sizeof(double));
    for (int i = 0; i < sizeof(double); i++, buf++)
        c_val[i] = *buf;
  
    double *a = (double*)c_val;
    *val = *a;
    return buf;
}

char *deserialize_vector3(vector3_t *v, char *buf)
{
    buf = deserialize_float_as_f64_little_endian(&v->x, buf);
    buf = deserialize_float_as_f64_little_endian(&v->y, buf);
    buf = deserialize_float_as_f64_little_endian(&v->z, buf);
  
    return buf;
}

char *deserialize_twist(twist_t *t, char *buf)
{
    // FIXME: check the little/big endian value
    *(buf++);
    *(buf++);
    *(buf++);
    *(buf++);
  
    buf = deserialize_vector3(&t->linear, buf);
    buf = deserialize_vector3(&t->angular, buf);
  
    return buf;
}

void commandVelocityCallback(const zn_sample_t *sample, const void *arg)
{
    (void)(arg); // Unused argument
    led_status = !led_status;

    twist_t cmd_vel_msg;
    deserialize_twist(&cmd_vel_msg, (char*)sample->value.val);
  
    Serial.println("commandVelocityCallback");
    Serial.print("  X:");
    Serial.println(cmd_vel_msg.linear.x);
    Serial.print("  Y:");
    Serial.println(cmd_vel_msg.angular.z);

    goal_velocity[VelocityType::LINEAR] = constrain((float)(cmd_vel_msg.linear.x), min_linear_velocity, max_linear_velocity);
    goal_velocity[VelocityType::ANGULAR] = constrain((float)(cmd_vel_msg.angular.z), min_angular_velocity, max_angular_velocity);
}

int wifiInit()
{
    // Required to remap interrupt pin with opencr
    WiFi.setPins(10, digitalPinToInterrupt(7), 5, -1); 
    
    if (WiFi.status() == WL_NO_SHIELD)
        return -1;
  
    WiFi.begin(SSID, PASS);
}

zn_session_t *zenohInit()
{
    zn_properties_t *config = zn_config_default();
    zn_properties_insert(config, ZN_CONFIG_MODE_KEY, z_string_make(MODE));
    if (strcmp(PEER, "") != 0)
        zn_properties_insert(config, ZN_CONFIG_PEER_KEY, z_string_make(PEER));
  
    zn_session_t *s = zn_open(config);
    
    return s;
}

void setup()
{
    Serial.begin(115200);
    delay(5000);

    // Initialize WiFi module and connect to network
    if (wifiInit() < 0)
    {
        Serial.println("WiFi shield not present");
        while (true);
    }
    Serial.println("WiFi Connected!");  

    // Establish zenoh session
    Serial.println("Session initializing...");
    zn = zenohInit();
    if (zn == NULL)
    {
        Serial.println("Error establishing zenoh session!"); 
        while(true);
    }
    Serial.println("Session initializing...done");

    zn_subscriber_t *cmd_vel_sub = zn_declare_subscriber(zn, zn_rname("/rt/cmd_vel"), zn_subinfo_default(), commandVelocityCallback, NULL);

    pinMode(BDPIN_LED_USER_4, OUTPUT);
    pinMode(BDPIN_LED_USER_1, OUTPUT);

    // Initialize motors drivers
    int ret = motor_driver.init();
    if (ret == false)
    {
        Serial.println("Failing to initialize the motor drivers");
        while(true);
    }

    Serial.println("Success to initialize to the motor drivers");
    delay(1000);
    if (motor_driver.is_connected() == false)
    {
        Serial.println("Failing to connect to the motor drivers");
        while(true);
    }
    motor_driver.set_torque(true);
    Serial.println("Successed to initialize and connect the motor drivers");

    max_linear_velocity = burger_info.wheel_radius * 2 * PI * burger_info.model_motor_rpm / 60;
    min_linear_velocity = -max_linear_velocity;
    max_angular_velocity = max_linear_velocity / burger_info.turning_radius;
    min_angular_velocity = -max_angular_velocity;
    digitalWrite(BDPIN_LED_USER_1, LOW);
}

void loop()
{
    Serial.println("Loop");

    // Blink on callback
    digitalWrite(BDPIN_LED_USER_4, led_status);

    // Read Zenoh Message
    znp_read(zn);

    // Send Keep Alive
    znp_send_keep_alive(zn);

    // Spin over the hardware
    motor_driver.control_motors(burger_info.wheel_separation, goal_velocity[VelocityType::LINEAR], goal_velocity[VelocityType::ANGULAR]);
}

