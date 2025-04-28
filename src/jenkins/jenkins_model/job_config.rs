use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FlowDefinition {
    #[serde(rename = "properties")]
    pub properties: FlowDefinitionProperties,
}

#[derive(Debug, Deserialize)]
struct FlowDefinitionProperties {
    #[serde(rename = "hudson.model.ParametersDefinitionProperty")]
    pub parameters_definition_property: ParameterDefinitions,
}

#[derive(Debug, Deserialize)]
struct ParameterDefinitions {
    #[serde(rename = "parameterDefinitions")]
    pub wrapper: ParameterDefinitionsWrapper,
}

#[derive(Debug, Deserialize)]
struct ParameterDefinitionsWrapper {
    #[serde(rename = "$value")]
    pub parameters: Vec<ParameterDefinition>,
}

#[derive(Debug, Deserialize)]
pub enum ParameterDefinition {
    #[serde(rename = "hudson.model.StringParameterDefinition")]
    StringParam {
        name: String,
        description: Option<String>,
        trim: Option<bool>,
        default_value: Option<String>,
    },
    #[serde(rename = "hudson.model.BooleanParameterDefinition")]
    BoolParam {
        name: String,
        description: Option<String>,
        default_value: Option<bool>,
    },
}

impl FlowDefinition {
    pub fn get_parameters(&self) -> &Vec<ParameterDefinition> {
        &self
            .properties
            .parameters_definition_property
            .wrapper
            .parameters
    }
}

#[cfg(test)]
mod tests {
    use crate::jenkins::jenkins_model::job_config::FlowDefinition;
    use serde::Deserialize;

    #[test]
    fn xml_de_lab() {
        #[derive(Debug, Deserialize)]
        struct TestContent {
            int_val: i32,
        }

        #[derive(Debug, Deserialize)]
        struct Test {
            str: String,

            content: TestContent,
        }

        let content = r#"<?xml version='1.1' encoding='UTF-8'?>
<Test>
    <str>hello world!</str>
        <content>
            <int_val>42</int_val>
        </content>
</Test>
"#;
        match quick_xml::de::from_str::<Test>(content) {
            Ok(result) => {
                println!("{:#?}", result);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_deserialize_config_xml() {
        let content = r#"<?xml version='1.1' encoding='UTF-8'?>
<flow-definition plugin="workflow-job@1385.vb_58b_86ea_fff1">
  <description></description>
  <keepDependencies>false</keepDependencies>
  <properties>
    <hudson.model.ParametersDefinitionProperty>
      <parameterDefinitions>
        <hudson.model.StringParameterDefinition>
          <name>Changelist</name>
          <trim>true</trim>
        </hudson.model.StringParameterDefinition>
        <hudson.model.StringParameterDefinition>
          <name>CustomServer</name>
          <description>Custom Server</description>
          <trim>true</trim>
        </hudson.model.StringParameterDefinition>
        <hudson.model.StringParameterDefinition>
          <name>ShelvedChange</name>
          <trim>true</trim>
        </hudson.model.StringParameterDefinition>
        <hudson.model.BooleanParameterDefinition>
          <name>ECP</name>
          <description>description of ECP</description>
          <defaultValue>false</defaultValue>
        </hudson.model.BooleanParameterDefinition>
        <hudson.model.BooleanParameterDefinition>
          <name>SAG</name>
          <description>description of SAG</description>
          <defaultValue>false</defaultValue>
        </hudson.model.BooleanParameterDefinition>
      </parameterDefinitions>
    </hudson.model.ParametersDefinitionProperty>
    <jenkins.model.BuildDiscarderProperty>
      <strategy class="hudson.tasks.LogRotator">
        <daysToKeep>-1</daysToKeep>
        <numToKeep>100</numToKeep>
        <artifactDaysToKeep>-1</artifactDaysToKeep>
        <artifactNumToKeep>-1</artifactNumToKeep>
      </strategy>
    </jenkins.model.BuildDiscarderProperty>
  </properties>
  <triggers/>
  <disabled>false</disabled>
</flow-definition>
"#;

        match quick_xml::de::from_str::<FlowDefinition>(content) {
            Ok(result) => {
                println!("{:#?}", result);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}
