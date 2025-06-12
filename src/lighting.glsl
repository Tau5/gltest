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


uniform Light light;
uniform Material material;

void main() {
    // Ambient
    bool spotLightLight = false;
    vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoord));

    vec3 norm = normalize(Normal);
    vec3 lightDir;
    float attenuation = 1.0;

    if (light.vector.w == 1.0) {
        // Light Point
        lightDir = normalize(light.vector.xyz - FragPos);

        if (light.spAngleInner > 0.0) {
            float angleInnerCone = dot(lightDir, normalize(-light.spotlightDirection));
            float angleOuterCone = light.spAngleInner - light.spAngleOuter;
            float intensity = clamp((angleInnerCone - light.spAngleOuter) / angleOuterCone, 0.0, 1.0);
            attenuation = intensity;
        } else {
            float distance = length(light.vector.xyz - FragPos);
            attenuation = 1.0 / (light.constant + light.linear * distance +
            light.quadratic * (distance * distance));
        }
    } else {
        lightDir = vec3(normalize(-light.vector));
    }

    // Diffuse
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoord));

    // Specular
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    vec3 specular = vec3(texture(material.specular, TexCoord)) * spec * light.specular;


    //vec3 result = (diffuse + ambient + specular) * objectColor;

    if (light.spAngleInner <= 0.0) {
        ambient *= attenuation;
    }
    diffuse *= attenuation;
    specular *= attenuation;

    vec3 result;

    result = (diffuse + ambient + specular + vec3(texture(material.emission, TexCoord)));

    FragColor = vec4(result, texture(material.diffuse, TexCoord).w);
}