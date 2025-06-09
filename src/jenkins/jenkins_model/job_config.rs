use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
pub struct FlowDefinition {
    #[serde(rename = "properties")]
    properties: FlowDefinitionProperties,
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
#[allow(dead_code)]
pub enum ParameterDefinition {
    #[serde(rename = "hudson.model.StringParameterDefinition")]
    String {
        name: String,
        description: Option<XmlRichText>,
        #[allow(dead_code)]
        trim: Option<bool>,
        #[serde(rename = "defaultValue")]
        default_value: Option<String>,
    },
    #[serde(rename = "hudson.model.BooleanParameterDefinition")]
    Bool {
        name: String,
        description: Option<XmlRichText>,
        #[serde(default, rename = "defaultValue")]
        default_value: bool,
    },
    #[serde(rename = "hudson.model.ChoiceParameterDefinition")]
    Choice {
        name: String,
        description: Option<XmlRichText>,
        #[serde(default)]
        choices: Vec<String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct XmlRichText {
    #[serde(rename = "$value")]
    pub content: Option<Vec<XmlRichTextElem>>,
}

impl Display for XmlRichText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let XmlRichText { content } = self;
        {
            if let Some(ref content) = content {
                for i in 0..content.len() {
                    write!(f, "{}", content[i])?;
                    if i < content.len() - 1 {
                        write!(f, " ")?;
                    }
                }
            }

            Ok(())
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum XmlRichTextElem {
    #[serde(rename = "$text")]
    Content(String),
    #[serde(rename = "span")]
    Span {
        style: Option<String>,
        #[serde(rename = "$text")]
        content: Option<String>,
    },
}

impl Display for XmlRichTextElem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlRichTextElem::Content(text) => write!(f, "{}", text),
            XmlRichTextElem::Span { style, content } => {
                if let Some(ref content) = content {
                    if let Some(ref style) = style {
                        write!(f, "<span style='{}'>{}</span>", style, content)
                    } else {
                        write!(f, "<span>{}</span>", content)
                    }
                } else {
                    Ok(())
                }
            }
        }
    }
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
#[allow(dead_code)]
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
                panic!();
            }
        }
    }

    #[test]
    fn test_deserialize_config_xml_with_rich_text_description() {
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
        <hudson.model.ChoiceParameterDefinition>
          <name>TestType</name>
          <description><span style='color:red'> Please select TestType before build.</span></description>
        </hudson.model.ChoiceParameterDefinition>
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
                panic!();
            }
        }
    }
    
    #[test]
    fn test_deserialize_config_xml_with_empty_description() {
        let content=r##"<flow-definition plugin="workflow-job@1385.vb_58b_86ea_fff1">
    <properties>
        <hudson.model.ParametersDefinitionProperty>
            <parameterDefinitions>
                <hudson.model.BooleanParameterDefinition>
                    <name>IsUseCore</name>
                    <description/>
                    <defaultValue>true</defaultValue>
                </hudson.model.BooleanParameterDefinition>
            </parameterDefinitions>
        </hudson.model.ParametersDefinitionProperty>
    </properties>
</flow-definition>"##;
        
        match quick_xml::de::from_str::<FlowDefinition>(content) {
            Ok(result) => {
                println!("{:#?}", result);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!();
            }
        }
    }
}
