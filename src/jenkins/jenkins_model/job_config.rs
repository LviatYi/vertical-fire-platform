use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize, PartialEq)]
pub struct FlowDefinition {
    properties: FlowDefinitionProperties,
}

#[derive(Debug, Deserialize, PartialEq)]
struct FlowDefinitionProperties {
    #[serde(rename = "hudson.model.ParametersDefinitionProperty")]
    pub parameters_definition_property: ParameterDefinitions,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ParameterDefinitions {
    #[serde(rename = "parameterDefinitions")]
    pub wrapper: ParameterDefinitionsWrapper,
}

#[derive(Debug, Deserialize, PartialEq)]
struct ParameterDefinitionsWrapper {
    #[serde(rename = "$value")]
    pub parameters: Vec<ParameterDefinition>,
}

#[derive(Debug, Deserialize, PartialEq)]
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
        choices: ChoiceParameterWrapper,
    },
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ChoiceParameterWrapper {
    #[serde(rename = "$value")]
    pub content: Vec<ChoiceParameter>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum ChoiceParameter {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "a")]
    Array(StringsValue),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct StringsValue {
    #[serde(rename = "string")]
    pub strings: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Deserialize, PartialEq)]
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
    use crate::jenkins::jenkins_model::job_config::*;
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
    fn xml_de_high_level_lab() {}

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
          <name>StringValueWithoutDesc</name>
          <trim>true</trim>
        </hudson.model.StringParameterDefinition>
        <hudson.model.StringParameterDefinition>
          <name>StringValueWithDesc</name>
          <description>description of StringValueWithDesc</description>
          <trim>true</trim>
        </hudson.model.StringParameterDefinition>
        <hudson.model.BooleanParameterDefinition>
          <name>SomeBoolValue1</name>
          <description>description of SomeBoolValue1</description>
          <defaultValue>true</defaultValue>
        </hudson.model.BooleanParameterDefinition>
        <hudson.model.BooleanParameterDefinition>
          <name>SomeBoolValue2</name>
          <description>description of SomeBoolValue2</description>
          <defaultValue>false</defaultValue>
        </hudson.model.BooleanParameterDefinition>
      </parameterDefinitions>
    </hudson.model.ParametersDefinitionProperty>
    <otherNode>
      <prop class="OtherNode">
        <someVal>-1</someVal>
        <someVal2>100</someVal2>
        <someVal3>-1</someVal3>
      </prop>
    </otherNode>
  </properties>
  <triggers/>
  <disabled>false</disabled>
</flow-definition>
"#;

        match quick_xml::de::from_str::<FlowDefinition>(content) {
            Ok(result) => {
                assert_eq!(
                    result
                        .properties
                        .parameters_definition_property
                        .wrapper
                        .parameters,
                    vec![
                        ParameterDefinition::String {
                            name: "StringValueWithoutDesc".to_string(),
                            description: None,
                            trim: Some(true),
                            default_value: None
                        },
                        ParameterDefinition::String {
                            name: "StringValueWithDesc".to_string(),
                            description: Some(XmlRichText {
                                content: Some(vec![XmlRichTextElem::Content(
                                    "description of StringValueWithDesc".to_string()
                                )])
                            }),
                            trim: Some(true),
                            default_value: None
                        },
                        ParameterDefinition::Bool {
                            name: "SomeBoolValue1".to_string(),
                            description: Some(XmlRichText {
                                content: Some(vec![XmlRichTextElem::Content(
                                    "description of SomeBoolValue1".to_string()
                                )]),
                            }),
                            default_value: true
                        },
                        ParameterDefinition::Bool {
                            name: "SomeBoolValue2".to_string(),
                            description: Some(XmlRichText {
                                content: Some(vec![XmlRichTextElem::Content(
                                    "description of SomeBoolValue2".to_string()
                                )]),
                            }),
                            default_value: false
                        },
                    ]
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!();
            }
        }
    }

    #[test]
    fn test_string_choice() {
        let content = r#"<?xml version='1.1' encoding='UTF-8'?>
<flow-definition plugin="workflow-job@1385.vb_58b_86ea_fff1">
  <description></description>
  <keepDependencies>false</keepDependencies>
  <properties>
    <hudson.model.ParametersDefinitionProperty>
      <parameterDefinitions>
        <hudson.model.ChoiceParameterDefinition>
          <name>Choice</name>
          <description><span style='color:red'>Some desc about choice type</span></description>
          <choices>
            <string/>
            <string>Content</string>
            <string>Other</string>
          </choices>
        </hudson.model.ChoiceParameterDefinition>
      </parameterDefinitions>
    </hudson.model.ParametersDefinitionProperty>
    <otherNode>
      <prop class="OtherNode">
        <someVal>-1</someVal>
        <someVal2>100</someVal2>
        <someVal3>-1</someVal3>
      </prop>
    </otherNode>
  </properties>
  <triggers/>
  <disabled>false</disabled>
</flow-definition>
"#;

        match quick_xml::de::from_str::<FlowDefinition>(content) {
            Ok(result) => {
                assert_eq!(
                    result
                        .properties
                        .parameters_definition_property
                        .wrapper
                        .parameters,
                    vec![ParameterDefinition::Choice {
                        name: "Choice".to_string(),
                        description: Some(XmlRichText {
                            content: Some(vec![XmlRichTextElem::Span {
                                style: None,
                                content: Some("Some desc about choice type".to_string())
                            }])
                        }),
                        choices: ChoiceParameterWrapper {
                            content: vec![
                                ChoiceParameter::String("".to_string()),
                                ChoiceParameter::String("Content".to_string()),
                                ChoiceParameter::String("Other".to_string())
                            ]
                        }
                    }]
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!();
            }
        }
    }

    #[test]
    fn test_array_choice() {
        let content = r#"<?xml version='1.1' encoding='UTF-8'?>
<flow-definition plugin="workflow-job@1385.vb_58b_86ea_fff1">
  <description></description>
  <keepDependencies>false</keepDependencies>
  <properties>
    <hudson.model.ParametersDefinitionProperty>
      <parameterDefinitions>
        <hudson.model.ChoiceParameterDefinition>
          <name>Choice</name>
          <description><span style='color:red'>Some desc about choice type</span></description>
          <choices class="java.util.Arrays$ArrayList">
            <a class="string-array">
              <string>hello</string>
              <string>world</string>
            </a>
          </choices>
        </hudson.model.ChoiceParameterDefinition>
      </parameterDefinitions>
    </hudson.model.ParametersDefinitionProperty>
    <otherNode>
      <prop class="OtherNode">
        <someVal>-1</someVal>
        <someVal2>100</someVal2>
        <someVal3>-1</someVal3>
      </prop>
    </otherNode>
  </properties>
  <triggers/>
  <disabled>false</disabled>
</flow-definition>
"#;

        match quick_xml::de::from_str::<FlowDefinition>(content) {
            Ok(result) => {
                assert_eq!(
                    result
                        .properties
                        .parameters_definition_property
                        .wrapper
                        .parameters,
                    vec![ParameterDefinition::Choice {
                        name: "Choice".to_string(),
                        description: Some(XmlRichText {
                            content: Some(vec![XmlRichTextElem::Span {
                                style: None,
                                content: Some("Some desc about choice type".to_string())
                            }])
                        }),
                        choices: ChoiceParameterWrapper {
                            content: vec![ChoiceParameter::Array(StringsValue {
                                strings: vec!["hello".to_string(), "world".to_string()]
                            }),]
                        }
                    }]
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!();
            }
        }
    }

    #[test]
    fn test_deserialize_config_xml_with_empty_description() {
        let content = r##"<flow-definition plugin="workflow-job@1385.vb_58b_86ea_fff1">
    <properties>
        <hudson.model.ParametersDefinitionProperty>
            <parameterDefinitions>
                <hudson.model.BooleanParameterDefinition>
                    <name>SomeBoolValue</name>
                    <description/>
                    <defaultValue>true</defaultValue>
                </hudson.model.BooleanParameterDefinition>
            </parameterDefinitions>
        </hudson.model.ParametersDefinitionProperty>
    </properties>
</flow-definition>"##;

        match quick_xml::de::from_str::<FlowDefinition>(content) {
            Ok(result) => {
                assert_eq!(
                    result
                        .properties
                        .parameters_definition_property
                        .wrapper
                        .parameters,
                    vec![ParameterDefinition::Bool {
                        name: "SomeBoolValue".to_string(),
                        description: Some(XmlRichText { content: None }),
                        default_value: true
                    }]
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!();
            }
        }

        let content = r##"<flow-definition plugin="workflow-job@1385.vb_58b_86ea_fff1">
    <properties>
        <hudson.model.ParametersDefinitionProperty>
            <parameterDefinitions>
                <hudson.model.BooleanParameterDefinition>
                    <name>SomeBoolValue</name>
                    <defaultValue>true</defaultValue>
                </hudson.model.BooleanParameterDefinition>
            </parameterDefinitions>
        </hudson.model.ParametersDefinitionProperty>
    </properties>
</flow-definition>"##;

        match quick_xml::de::from_str::<FlowDefinition>(content) {
            Ok(result) => {
                assert_eq!(
                    result
                        .properties
                        .parameters_definition_property
                        .wrapper
                        .parameters,
                    vec![ParameterDefinition::Bool {
                        name: "SomeBoolValue".to_string(),
                        description: None,
                        default_value: true
                    }]
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!();
            }
        }
    }
}
