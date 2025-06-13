#version 460 core
out vec4 FragColor;
in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoord;
uniform vec3 viewPos;

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    sampler2D emission;
    float shininess;
};

/*
struct Light {
    vec4 vector;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;

    float spAngleInner;
    float spAngleOuter;
    vec3 spotlightDirection;
};
*/

struct DirLight {
    vec3 direction;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;

    float constant;
    float linear;
    float quadratic;
};

struct SpotLight {
    vec3 position;
    vec3 direction;
    float angleInner;
    float angleOuter;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform Material material;

uniform DirLight directionalLight;
uniform SpotLight spotLight;

// Like the olden' days
#define NR_POINT_LIGHTS 4
uniform PointLight pointLights[NR_POINT_LIGHTS];
uniform int lightpoint_num;

uniform bool spotLightEnable = false;

vec3 calculateDirLight(DirLight light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light.direction);

    // Diffuse
    float diff = max(dot(normal, lightDir), 0.0);
    // Specular
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);

    vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoord));
    vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoord));
    vec3 specular = vec3(texture(material.specular, TexCoord)) * spec * light.specular;

    vec3 result = (diffuse + ambient + specular + vec3(texture(material.emission, TexCoord)));

    return result;
}

vec3 calculatePointLight(PointLight light, vec3 normal, vec3 viewDir) {
    float attenuation = 1.0;
    vec3 lightDir = normalize(light.position - FragPos);

    // Diffuse
    float diff = max(dot(normal, lightDir), 0.0);
    // Specular
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);

    vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoord));
    vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoord));
    vec3 specular = vec3(texture(material.specular, TexCoord)) * spec * light.specular;

    float distance = length(light.position - FragPos);
    attenuation = 1.0 / (light.constant + light.linear * distance +
    light.quadratic * (distance * distance));

    ambient *= attenuation;
    diffuse *= attenuation;
    specular *= attenuation;

    vec3 result;

    result = (diffuse + ambient + specular + vec3(texture(material.emission, TexCoord)));

    return result;
}

vec3 calculateSpotlight(SpotLight light, vec3 normal, vec3 viewDir) {
    float attenuation = 1.0;
    vec3 lightDir = normalize(light.position - FragPos);

    float angleInnerCone = dot(lightDir, normalize(-light.direction));
    float angleOuterCone = light.angleInner - light.angleOuter;
    float intensity = clamp((angleInnerCone - light.angleOuter) / angleOuterCone, 0.0, 1.0);
    attenuation = intensity;

    // Diffuse
    float diff = max(dot(normal, lightDir), 0.0);
    // Specular
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);

    vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoord));
    vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoord));
    vec3 specular = vec3(texture(material.specular, TexCoord)) * spec * light.specular;

    diffuse *= attenuation;
    specular *= attenuation;

    return vec3(diffuse + ambient + specular + vec3(texture(material.emission, TexCoord)));
}


void main() {
    vec3 normal = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);

    vec3 result;
    result = calculateDirLight(directionalLight, normal, viewDir);

    for (int i = 0; i < NR_POINT_LIGHTS && i < lightpoint_num; i++)
        result += calculatePointLight(pointLights[i], normal, viewDir);

    if (spotLightEnable) {
        result += calculateSpotlight(spotLight, normal, viewDir);
    }

    FragColor = vec4(result, texture(material.diffuse, TexCoord).w);
}